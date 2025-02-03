package com.muuzika

import com.muuzika.common.RoomCode
import com.muuzika.registry.*
import org.apache.pekko.actor.typed.ActorSystem
import org.apache.pekko.stream.scaladsl.SourceQueueWithComplete
import org.slf4j.LoggerFactory

import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.atomic.AtomicInteger
import scala.concurrent.ExecutionContext.Implicits.global
import scala.concurrent.Future
import scala.jdk.CollectionConverters.*

class Room(val code: RoomCode, val server: Server) {
  server.rooms.put(code, this)
}

class Registry(system: ActorSystem[?], val codeGenerator: CodeGenerator = new CodeGenerator(None, 4, 100)) {
  private val rooms = new ConcurrentHashMap[RoomCode, Room]()
  private val servers = new ConcurrentHashMap[ServerId, Server]()
  private val serverIdCounter = new AtomicInteger(0)
  private val log = LoggerFactory.getLogger(getClass)

  def addServer(registration: ServerRegistrationRequest, queue: SourceQueueWithComplete[RegistryToServerMessage]): (Server, ServerRegistrationResponse) = {
    val serverId = ServerId(serverIdCounter.getAndIncrement())
    val server = new Server(system, serverId, registration.callsign, registration.address, queue)

    val conflicts = registration.rooms.flatMap(room => {
      room.code.flatMap(code => {
        var alreadyExists = true

        rooms.computeIfAbsent(code, _ => {
          alreadyExists = false
          new Room(code, server)
        })

        if (alreadyExists) {
          val newCode = getRoomCode
          rooms.put(newCode, new Room(newCode, server))
          Some(RoomCodeChange(before = Some(code), after = Some(newCode)))
        } else {
          None
        }
      })
    })
    servers.put(serverId, server)

    (server, ServerRegistrationResponse(serverId = Some(serverId), conflicts = conflicts))
  }

  def removeServer(server: Server): Unit = {
    log.info("Removing server {} with {} rooms", server.id, server.roomCount)
    servers.remove(server.id)
    server.rooms.values.asScala.foreach(room => rooms.remove(room.code))
    server.shutdown()
  }

  def createRoom(request: CreateRoomRequest): Future[CreateRoomResponse] = {
    getLeastLoadedServer match {
      // TODO: custom exception class
      case None => Future.failed(new Exception("No servers available"))
      case Some(server) =>
        val code = getRoomCode
        server.createRoom(code, request).map(_ => {
          val room = new Room(code, server)
          rooms.put(code, room)
          CreateRoomResponse(
            code = Some(code),
            server = Some(server.identifier)
          )
        })
    }
  }

  private def getLeastLoadedServer: Option[Server] = servers.values.asScala.minByOption(_.roomCount)

  private def getRoomCode: RoomCode = {
    val (code, isEmpty) = codeGenerator.getCode
    if (isEmpty) Future {
      codeGenerator.createNextCollection
    }
    code
  }

  def getRoom(code: RoomCode): Option[Room] = Option(rooms.get(code))
}

package com.muuzika

import com.muuzika.common.RoomCode
import com.muuzika.Room
import com.muuzika.registry.{CreateRoomInServerRequest, CreateRoomRequest, RegistryToServerMessage, RegistryToServerRequest, RegistryToServerResponse, ServerId, ServerIdentifier, ServerToRegistryMessage, ServerToRegistryResponse, ServerToRegistrySuccess}
import org.apache.pekko.actor.typed.{ActorRef, ActorSystem}

import java.util.concurrent.ConcurrentHashMap
import scala.concurrent.{Future, Promise}
import com.muuzika.ProtoExtensions.*
import org.apache.pekko.stream.scaladsl.SourceQueueWithComplete
import org.slf4j.LoggerFactory

import scala.concurrent.ExecutionContext.Implicits.global
import java.util.concurrent.atomic.AtomicLong

class Server(system: ActorSystem[?], val id: ServerId, val callsign: String, val address: String, val queue: SourceQueueWithComplete[RegistryToServerMessage]) {
  private implicit val sys: ActorSystem[?] = system

  val rooms = new ConcurrentHashMap[RoomCode, Room]()
  val identifier: ServerIdentifier = ServerIdentifier(address = address)

  private val requestIdCounter = new AtomicLong(0)
  private val promises = new ConcurrentHashMap[Long, Promise[ServerToRegistrySuccess.Success]]()

  private val log = LoggerFactory.getLogger(s"${getClass.getName}[${id.serverId}.$callsign]")

  def roomCount: Int = rooms.size

  def handleMessage(message: ServerToRegistryMessage): Unit = {
    log.debug("Received message: {}", message)
    message.message match {
      case ServerToRegistryMessage.Message.Response(response) =>
        message.requestId.fold(log.warn("Received response without request id"))(handleResponse(_, response))
      case _ => ()
    }
  }

  def createRoom(code: RoomCode, request: CreateRoomRequest): Future[Unit] = {
    val (requestId, future) = createRequestFuture()
    
    val serverRequest = CreateRoomInServerRequest(
      code = Some(code),
      leaderUsername = request.leaderUsername,
      isPublic = request.isPublic,
      password = request.password
    )

    for {
      _ <- queue.offer(RegistryToServerRequest.Request.CreateRoom(serverRequest).pack(requestId))
      _ <- future
    } yield ()
  }

  def shutdown(): Unit = {
    queue.complete()
    promises.values().forEach(_.failure(new Exception("Server shutdown")))
  }

  private def createRequestFuture(): (Long, Future[ServerToRegistrySuccess.Success]) = {
    val requestId = requestIdCounter.incrementAndGet()
    val promise = Promise[ServerToRegistrySuccess.Success]()
    promises.put(requestId, promise)
    (requestId, promise.future)
  }

  private def handleResponse(requestId: Long, response: ServerToRegistryResponse): Unit = {
    import ServerToRegistryResponse.Response._

    val promise = promises.remove(requestId)
    if (promise != null) {
      response.response match {
        case Success(success) =>
          log.debug("Received success response for request with id={}", requestId)
          promise.success(success.success)

        case Error(error) =>
          log.warn("Received error response for request with id={}", requestId)
          promise.failure(error.error.asThrowable)

        case Empty =>
          log.warn("Received empty response for request with id={}", requestId)
          promise.failure(new Exception("Empty response"))

      }
    }

  }

}

sealed trait ServerOutbound {}
object ServerOutbound {
  case class Send(message: com.muuzika.registry.RegistryToServerMessage) extends ServerOutbound
  case class Complete() extends ServerOutbound
  case class Fail(ex: Throwable) extends ServerOutbound
}

package com.muuzika.services

import com.muuzika.ProtoExtensions.*
import com.muuzika.registry.*
import com.muuzika.Registry

import org.apache.pekko.actor.typed.ActorSystem
import org.slf4j.LoggerFactory

import scala.concurrent.Future

class RegistryServiceLobbyImpl(system: ActorSystem[?], registry: Registry) extends RegistryLobbyService {
  private implicit val sys: ActorSystem[?] = system
  private val log = LoggerFactory.getLogger(getClass)

  def createRoom(request: CreateRoomRequest): Future[CreateRoomResponse] = {
    registry.createRoom(request)
  }
}

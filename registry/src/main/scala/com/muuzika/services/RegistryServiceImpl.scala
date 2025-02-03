package com.muuzika.services

import com.muuzika.ProtoExtensions.*
import com.muuzika.Registry
import com.muuzika.registry.*
import io.grpc.Status
import org.apache.pekko.NotUsed
import org.apache.pekko.actor.typed.ActorSystem
import org.apache.pekko.grpc.GrpcServiceException
import org.apache.pekko.stream.{KillSwitches, OverflowStrategy}
import org.apache.pekko.stream.scaladsl.{Keep, Sink, Source}
import org.slf4j.LoggerFactory

import scala.concurrent.ExecutionContext.Implicits.global
import scala.concurrent.duration.*
import scala.concurrent.{Await, TimeoutException}
import scala.util.{Failure, Success, Try}

class RegistryServiceImpl(system: ActorSystem[?], registry: Registry) extends RegistryService {
  private implicit val sys: ActorSystem[?] = system
  private val registrationTimeout = 5.seconds
  private val log = LoggerFactory.getLogger(getClass)

  override def registerServer(in: Source[ServerToRegistryMessage, NotUsed]): Source[RegistryToServerMessage, NotUsed] = waitForRegistration(in)
    .fold(ex => {
      log.warn("Failed to capture registration for server: {}", ex)
      Source.failed(ex)
    }, { case (rest, requestId, registration) =>
      log.info("Received registration for server, callsign={}, address={}", registration.callsign, registration.address)

      val (queue, source) = Source.queue[RegistryToServerMessage](8, OverflowStrategy.backpressure)
        .preMaterialize()

      val (killSwitch, controlledIn) = rest.viaMat(KillSwitches.single)(Keep.right).preMaterialize()
      queue.watchCompletion().andThen(_ => killSwitch.shutdown())

      val (server, response) = registry.addServer(registration, queue)

      queue.offer(RegistryToServerSuccess.Success.RegistrationResponse(response).pack(requestId))

      controlledIn.runForeach(server.handleMessage)
        .andThen { _ => registry.removeServer(server) }
        .onComplete {
          case Success(_) => queue.complete()
          case Failure(ex) => queue.fail(ex)
        }

      source
    })

  private def waitForRegistration(in: Source[ServerToRegistryMessage, NotUsed]): Try[(Source[ServerToRegistryMessage, NotUsed], Long, ServerRegistrationRequest)] = {
    Try(Await.result(in.prefixAndTail(1).runWith(Sink.head), registrationTimeout))
      .flatMap((seq, rest) => {
        extractRegistration(seq.head)
          .map((requestId, registration) => (rest, requestId, registration))
          .toRight(new GrpcServiceException(Status.INVALID_ARGUMENT.withDescription("First message was not a valid registration message"))).toTry
      })
      .recoverWith {
        case ex: GrpcServiceException => Failure(ex)
        case _: TimeoutException =>
          Failure(new GrpcServiceException(Status.DEADLINE_EXCEEDED.withDescription("Timeout for registration exceeded")))
        case ex: Exception =>
          Failure(new GrpcServiceException(Status.INTERNAL.withDescription(ex.getMessage)))
      }
  }

  private def extractRegistration(message: ServerToRegistryMessage): Option[(Long, ServerRegistrationRequest)] = {
    message.getRequest.request match {
      case ServerToRegistryRequest.Request.Registration(registration) => message.requestId.map(id => (id, registration))
      case _ => None
    }
  }


}

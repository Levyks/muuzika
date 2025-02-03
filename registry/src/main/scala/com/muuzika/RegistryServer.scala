package com.muuzika

import com.muuzika.registry.{RegistryLobbyServiceHandler, RegistryServiceHandler}
import com.muuzika.services.{RegistryServiceImpl, RegistryServiceLobbyImpl}
import com.typesafe.config.ConfigFactory
import org.apache.pekko.actor.typed.ActorSystem
import org.apache.pekko.actor.typed.scaladsl.Behaviors
import org.apache.pekko.http.scaladsl.Http
import org.apache.pekko.http.scaladsl.model.{HttpRequest, HttpResponse}
import org.slf4j.LoggerFactory
import org.apache.pekko.grpc.scaladsl.ServiceHandler

import java.lang.reflect.Field
import java.util.concurrent.ForkJoinPool
import scala.concurrent.{ExecutionContext, Future}
import scala.util.{Failure, Success}
import scala.concurrent.duration.*

object RegistryServer {
  def main(args: Array[String]): Unit = {
    val conf = ConfigFactory.parseString("pekko.http.server.preview.enable-http2 = on")
      .withFallback(ConfigFactory.defaultApplication())
    val system = ActorSystem[Nothing](Behaviors.empty[Nothing], "RegistryServer", conf)
    new RegistryServer(system).run()
  }
}

class RegistryServer(system: ActorSystem[?]) {
  private val log = LoggerFactory.getLogger(getClass)

  private def run(): Future[Http.ServerBinding] = {
    implicit val sys: ActorSystem[?] = system
    implicit val ec: ExecutionContext = system.executionContext
    
    val registry = new Registry(system)

    val registryLobbyService = RegistryLobbyServiceHandler.partial(new RegistryServiceLobbyImpl(system, registry))
    val registryService = RegistryServiceHandler.partial(new RegistryServiceImpl(system, registry))

    val bound: Future[Http.ServerBinding] = Http()(system)
      .newServerAt(interface = "127.0.0.1", port = 8080)
      .bind(ServiceHandler.concatOrNotFound(registryLobbyService, registryService))
      .map(_.addToCoordinatedShutdown(hardTerminationDeadline = 10.seconds))

    bound.onComplete {
      case Success(binding) =>
        val address = binding.localAddress
        log.info(s"gRPC server bound to {}:{}", address.getHostString, address.getPort)
      case Failure(ex) =>
        log.error("Failed to bind gRPC endpoint, terminating system", ex)
        system.terminate()
    }

    bound
  }
}
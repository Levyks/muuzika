package com.muuzika

import com.muuzika.registry.{RegistryToServerMessage, RegistryToServerRequest, RegistryToServerResponse, RegistryToServerSuccess, ServerToRegistryError}
import com.muuzika.Server

object ProtoExtensions {
  implicit class ExtendedRegistryToServerSuccess(val success: RegistryToServerSuccess.Success) {
    def pack(requestId: Long): RegistryToServerMessage = RegistryToServerMessage(
      requestId = Some(requestId),
      message = RegistryToServerMessage.Message.Response(
        RegistryToServerResponse(
          response = RegistryToServerResponse.Response.Success(RegistryToServerSuccess(success))
        )
      )
    )
  }

  implicit class ExtendedRegistryToServerRequest(val request: RegistryToServerRequest.Request) {
    def pack(requestId: Long): RegistryToServerMessage = RegistryToServerMessage(
      requestId = Some(requestId),
      message = RegistryToServerMessage.Message.Request(RegistryToServerRequest(request))
    )
  }
  
  implicit class ExtendedServerToRegistryError(val error: ServerToRegistryError.Error) {
    def asThrowable: ServerToRegistryErrorThrowable = new ServerToRegistryErrorThrowable(error)
  }
  
  class ServerToRegistryErrorThrowable(val error: ServerToRegistryError.Error) extends Throwable
}

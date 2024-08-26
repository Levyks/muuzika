import {GrpcTransport} from "@protobuf-ts/grpc-transport";
import {ChannelCredentials} from "@grpc/grpc-js";
import {ConnectionHandlerServiceClient} from "../compiled/connection_handler_pb.client.js";

const transport = new GrpcTransport({
    host: "localhost:50051",
    channelCredentials: ChannelCredentials.createInsecure(),
});

export const client = new ConnectionHandlerServiceClient(transport);
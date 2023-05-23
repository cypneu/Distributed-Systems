# Generated by the gRPC Python protocol compiler plugin. DO NOT EDIT!
"""Client and server classes corresponding to protobuf-defined services."""
import grpc

from proto import subscription_pb2 as proto_dot_subscription__pb2


class SubscriptionStub(object):
    """Missing associated documentation comment in .proto file."""

    def __init__(self, channel):
        """Constructor.

        Args:
            channel: A grpc.Channel.
        """
        self.Subscribe = channel.stream_stream(
                '/subscription.Subscription/Subscribe',
                request_serializer=proto_dot_subscription__pb2.SubscriptionRequest.SerializeToString,
                response_deserializer=proto_dot_subscription__pb2.Notification.FromString,
                )


class SubscriptionServicer(object):
    """Missing associated documentation comment in .proto file."""

    def Subscribe(self, request_iterator, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')


def add_SubscriptionServicer_to_server(servicer, server):
    rpc_method_handlers = {
            'Subscribe': grpc.stream_stream_rpc_method_handler(
                    servicer.Subscribe,
                    request_deserializer=proto_dot_subscription__pb2.SubscriptionRequest.FromString,
                    response_serializer=proto_dot_subscription__pb2.Notification.SerializeToString,
            ),
    }
    generic_handler = grpc.method_handlers_generic_handler(
            'subscription.Subscription', rpc_method_handlers)
    server.add_generic_rpc_handlers((generic_handler,))


 # This class is part of an EXPERIMENTAL API.
class Subscription(object):
    """Missing associated documentation comment in .proto file."""

    @staticmethod
    def Subscribe(request_iterator,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.stream_stream(request_iterator, target, '/subscription.Subscription/Subscribe',
            proto_dot_subscription__pb2.SubscriptionRequest.SerializeToString,
            proto_dot_subscription__pb2.Notification.FromString,
            options, channel_credentials,
            insecure, call_credentials, compression, wait_for_ready, timeout, metadata)

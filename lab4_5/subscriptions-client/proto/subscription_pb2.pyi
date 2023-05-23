from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Notification(_message.Message):
    __slots__ = ["content", "frequency", "notification_type"]
    class NotificationType(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
        __slots__ = []
    class Content(_message.Message):
        __slots__ = ["body"]
        BODY_FIELD_NUMBER: _ClassVar[int]
        body: _containers.RepeatedScalarFieldContainer[str]
        def __init__(self, body: _Optional[_Iterable[str]] = ...) -> None: ...
    ADVERTISEMENT: Notification.NotificationType
    CONTENT_FIELD_NUMBER: _ClassVar[int]
    FREQUENCY_FIELD_NUMBER: _ClassVar[int]
    INFORMATION: Notification.NotificationType
    NOTIFICATION_TYPE_FIELD_NUMBER: _ClassVar[int]
    content: Notification.Content
    frequency: int
    notification_type: Notification.NotificationType
    def __init__(self, frequency: _Optional[int] = ..., notification_type: _Optional[_Union[Notification.NotificationType, str]] = ..., content: _Optional[_Union[Notification.Content, _Mapping]] = ...) -> None: ...

class SubscriptionRequest(_message.Message):
    __slots__ = ["name", "notification_frequency"]
    NAME_FIELD_NUMBER: _ClassVar[int]
    NOTIFICATION_FREQUENCY_FIELD_NUMBER: _ClassVar[int]
    name: str
    notification_frequency: int
    def __init__(self, name: _Optional[str] = ..., notification_frequency: _Optional[int] = ...) -> None: ...

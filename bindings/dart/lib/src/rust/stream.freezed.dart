// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'stream.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AccountStreamItem {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountStreamItem);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AccountStreamItem()';
}


}

/// @nodoc
class $AccountStreamItemCopyWith<$Res>  {
$AccountStreamItemCopyWith(AccountStreamItem _, $Res Function(AccountStreamItem) __);
}


/// Adds pattern-matching-related methods to [AccountStreamItem].
extension AccountStreamItemPatterns on AccountStreamItem {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( AccountStreamItem_Event value)?  event,TResult Function( AccountStreamItem_Error value)?  error,TResult Function( AccountStreamItem_End value)?  end,required TResult orElse(),}){
final _that = this;
switch (_that) {
case AccountStreamItem_Event() when event != null:
return event(_that);case AccountStreamItem_Error() when error != null:
return error(_that);case AccountStreamItem_End() when end != null:
return end(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( AccountStreamItem_Event value)  event,required TResult Function( AccountStreamItem_Error value)  error,required TResult Function( AccountStreamItem_End value)  end,}){
final _that = this;
switch (_that) {
case AccountStreamItem_Event():
return event(_that);case AccountStreamItem_Error():
return error(_that);case AccountStreamItem_End():
return end(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( AccountStreamItem_Event value)?  event,TResult? Function( AccountStreamItem_Error value)?  error,TResult? Function( AccountStreamItem_End value)?  end,}){
final _that = this;
switch (_that) {
case AccountStreamItem_Event() when event != null:
return event(_that);case AccountStreamItem_Error() when error != null:
return error(_that);case AccountStreamItem_End() when end != null:
return end(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( WireAccountEvent field0)?  event,TResult Function( NativeError field0)?  error,TResult Function()?  end,required TResult orElse(),}) {final _that = this;
switch (_that) {
case AccountStreamItem_Event() when event != null:
return event(_that.field0);case AccountStreamItem_Error() when error != null:
return error(_that.field0);case AccountStreamItem_End() when end != null:
return end();case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( WireAccountEvent field0)  event,required TResult Function( NativeError field0)  error,required TResult Function()  end,}) {final _that = this;
switch (_that) {
case AccountStreamItem_Event():
return event(_that.field0);case AccountStreamItem_Error():
return error(_that.field0);case AccountStreamItem_End():
return end();}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( WireAccountEvent field0)?  event,TResult? Function( NativeError field0)?  error,TResult? Function()?  end,}) {final _that = this;
switch (_that) {
case AccountStreamItem_Event() when event != null:
return event(_that.field0);case AccountStreamItem_Error() when error != null:
return error(_that.field0);case AccountStreamItem_End() when end != null:
return end();case _:
  return null;

}
}

}

/// @nodoc


class AccountStreamItem_Event extends AccountStreamItem {
  const AccountStreamItem_Event(this.field0): super._();


 final  WireAccountEvent field0;

/// Create a copy of AccountStreamItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AccountStreamItem_EventCopyWith<AccountStreamItem_Event> get copyWith => _$AccountStreamItem_EventCopyWithImpl<AccountStreamItem_Event>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountStreamItem_Event&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'AccountStreamItem.event(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AccountStreamItem_EventCopyWith<$Res> implements $AccountStreamItemCopyWith<$Res> {
  factory $AccountStreamItem_EventCopyWith(AccountStreamItem_Event value, $Res Function(AccountStreamItem_Event) _then) = _$AccountStreamItem_EventCopyWithImpl;
@useResult
$Res call({
 WireAccountEvent field0
});


$WireAccountEventCopyWith<$Res> get field0;

}
/// @nodoc
class _$AccountStreamItem_EventCopyWithImpl<$Res>
    implements $AccountStreamItem_EventCopyWith<$Res> {
  _$AccountStreamItem_EventCopyWithImpl(this._self, this._then);

  final AccountStreamItem_Event _self;
  final $Res Function(AccountStreamItem_Event) _then;

/// Create a copy of AccountStreamItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AccountStreamItem_Event(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireAccountEvent,
  ));
}

/// Create a copy of AccountStreamItem
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$WireAccountEventCopyWith<$Res> get field0 {

  return $WireAccountEventCopyWith<$Res>(_self.field0, (value) {
    return _then(_self.copyWith(field0: value));
  });
}
}

/// @nodoc


class AccountStreamItem_Error extends AccountStreamItem {
  const AccountStreamItem_Error(this.field0): super._();


 final  NativeError field0;

/// Create a copy of AccountStreamItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AccountStreamItem_ErrorCopyWith<AccountStreamItem_Error> get copyWith => _$AccountStreamItem_ErrorCopyWithImpl<AccountStreamItem_Error>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountStreamItem_Error&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'AccountStreamItem.error(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AccountStreamItem_ErrorCopyWith<$Res> implements $AccountStreamItemCopyWith<$Res> {
  factory $AccountStreamItem_ErrorCopyWith(AccountStreamItem_Error value, $Res Function(AccountStreamItem_Error) _then) = _$AccountStreamItem_ErrorCopyWithImpl;
@useResult
$Res call({
 NativeError field0
});




}
/// @nodoc
class _$AccountStreamItem_ErrorCopyWithImpl<$Res>
    implements $AccountStreamItem_ErrorCopyWith<$Res> {
  _$AccountStreamItem_ErrorCopyWithImpl(this._self, this._then);

  final AccountStreamItem_Error _self;
  final $Res Function(AccountStreamItem_Error) _then;

/// Create a copy of AccountStreamItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AccountStreamItem_Error(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as NativeError,
  ));
}


}

/// @nodoc


class AccountStreamItem_End extends AccountStreamItem {
  const AccountStreamItem_End(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountStreamItem_End);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AccountStreamItem.end()';
}


}




/// @nodoc
mixin _$MarketStreamItem {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MarketStreamItem);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'MarketStreamItem()';
}


}

/// @nodoc
class $MarketStreamItemCopyWith<$Res>  {
$MarketStreamItemCopyWith(MarketStreamItem _, $Res Function(MarketStreamItem) __);
}


/// Adds pattern-matching-related methods to [MarketStreamItem].
extension MarketStreamItemPatterns on MarketStreamItem {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( MarketStreamItem_Event value)?  event,TResult Function( MarketStreamItem_Error value)?  error,TResult Function( MarketStreamItem_End value)?  end,required TResult orElse(),}){
final _that = this;
switch (_that) {
case MarketStreamItem_Event() when event != null:
return event(_that);case MarketStreamItem_Error() when error != null:
return error(_that);case MarketStreamItem_End() when end != null:
return end(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( MarketStreamItem_Event value)  event,required TResult Function( MarketStreamItem_Error value)  error,required TResult Function( MarketStreamItem_End value)  end,}){
final _that = this;
switch (_that) {
case MarketStreamItem_Event():
return event(_that);case MarketStreamItem_Error():
return error(_that);case MarketStreamItem_End():
return end(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( MarketStreamItem_Event value)?  event,TResult? Function( MarketStreamItem_Error value)?  error,TResult? Function( MarketStreamItem_End value)?  end,}){
final _that = this;
switch (_that) {
case MarketStreamItem_Event() when event != null:
return event(_that);case MarketStreamItem_Error() when error != null:
return error(_that);case MarketStreamItem_End() when end != null:
return end(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( WireMarketEvent field0)?  event,TResult Function( NativeError field0)?  error,TResult Function()?  end,required TResult orElse(),}) {final _that = this;
switch (_that) {
case MarketStreamItem_Event() when event != null:
return event(_that.field0);case MarketStreamItem_Error() when error != null:
return error(_that.field0);case MarketStreamItem_End() when end != null:
return end();case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( WireMarketEvent field0)  event,required TResult Function( NativeError field0)  error,required TResult Function()  end,}) {final _that = this;
switch (_that) {
case MarketStreamItem_Event():
return event(_that.field0);case MarketStreamItem_Error():
return error(_that.field0);case MarketStreamItem_End():
return end();}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( WireMarketEvent field0)?  event,TResult? Function( NativeError field0)?  error,TResult? Function()?  end,}) {final _that = this;
switch (_that) {
case MarketStreamItem_Event() when event != null:
return event(_that.field0);case MarketStreamItem_Error() when error != null:
return error(_that.field0);case MarketStreamItem_End() when end != null:
return end();case _:
  return null;

}
}

}

/// @nodoc


class MarketStreamItem_Event extends MarketStreamItem {
  const MarketStreamItem_Event(this.field0): super._();


 final  WireMarketEvent field0;

/// Create a copy of MarketStreamItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$MarketStreamItem_EventCopyWith<MarketStreamItem_Event> get copyWith => _$MarketStreamItem_EventCopyWithImpl<MarketStreamItem_Event>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MarketStreamItem_Event&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'MarketStreamItem.event(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $MarketStreamItem_EventCopyWith<$Res> implements $MarketStreamItemCopyWith<$Res> {
  factory $MarketStreamItem_EventCopyWith(MarketStreamItem_Event value, $Res Function(MarketStreamItem_Event) _then) = _$MarketStreamItem_EventCopyWithImpl;
@useResult
$Res call({
 WireMarketEvent field0
});


$WireMarketEventCopyWith<$Res> get field0;

}
/// @nodoc
class _$MarketStreamItem_EventCopyWithImpl<$Res>
    implements $MarketStreamItem_EventCopyWith<$Res> {
  _$MarketStreamItem_EventCopyWithImpl(this._self, this._then);

  final MarketStreamItem_Event _self;
  final $Res Function(MarketStreamItem_Event) _then;

/// Create a copy of MarketStreamItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(MarketStreamItem_Event(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireMarketEvent,
  ));
}

/// Create a copy of MarketStreamItem
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$WireMarketEventCopyWith<$Res> get field0 {

  return $WireMarketEventCopyWith<$Res>(_self.field0, (value) {
    return _then(_self.copyWith(field0: value));
  });
}
}

/// @nodoc


class MarketStreamItem_Error extends MarketStreamItem {
  const MarketStreamItem_Error(this.field0): super._();


 final  NativeError field0;

/// Create a copy of MarketStreamItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$MarketStreamItem_ErrorCopyWith<MarketStreamItem_Error> get copyWith => _$MarketStreamItem_ErrorCopyWithImpl<MarketStreamItem_Error>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MarketStreamItem_Error&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'MarketStreamItem.error(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $MarketStreamItem_ErrorCopyWith<$Res> implements $MarketStreamItemCopyWith<$Res> {
  factory $MarketStreamItem_ErrorCopyWith(MarketStreamItem_Error value, $Res Function(MarketStreamItem_Error) _then) = _$MarketStreamItem_ErrorCopyWithImpl;
@useResult
$Res call({
 NativeError field0
});




}
/// @nodoc
class _$MarketStreamItem_ErrorCopyWithImpl<$Res>
    implements $MarketStreamItem_ErrorCopyWith<$Res> {
  _$MarketStreamItem_ErrorCopyWithImpl(this._self, this._then);

  final MarketStreamItem_Error _self;
  final $Res Function(MarketStreamItem_Error) _then;

/// Create a copy of MarketStreamItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(MarketStreamItem_Error(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as NativeError,
  ));
}


}

/// @nodoc


class MarketStreamItem_End extends MarketStreamItem {
  const MarketStreamItem_End(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MarketStreamItem_End);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'MarketStreamItem.end()';
}


}




/// @nodoc
mixin _$WireAccountEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireAccountEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireAccountEvent()';
}


}

/// @nodoc
class $WireAccountEventCopyWith<$Res>  {
$WireAccountEventCopyWith(WireAccountEvent _, $Res Function(WireAccountEvent) __);
}


/// Adds pattern-matching-related methods to [WireAccountEvent].
extension WireAccountEventPatterns on WireAccountEvent {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireAccountEvent_Balance value)?  balance,TResult Function( WireAccountEvent_Order value)?  order,TResult Function( WireAccountEvent_Reconnected value)?  reconnected,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireAccountEvent_Balance() when balance != null:
return balance(_that);case WireAccountEvent_Order() when order != null:
return order(_that);case WireAccountEvent_Reconnected() when reconnected != null:
return reconnected(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireAccountEvent_Balance value)  balance,required TResult Function( WireAccountEvent_Order value)  order,required TResult Function( WireAccountEvent_Reconnected value)  reconnected,}){
final _that = this;
switch (_that) {
case WireAccountEvent_Balance():
return balance(_that);case WireAccountEvent_Order():
return order(_that);case WireAccountEvent_Reconnected():
return reconnected(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireAccountEvent_Balance value)?  balance,TResult? Function( WireAccountEvent_Order value)?  order,TResult? Function( WireAccountEvent_Reconnected value)?  reconnected,}){
final _that = this;
switch (_that) {
case WireAccountEvent_Balance() when balance != null:
return balance(_that);case WireAccountEvent_Order() when order != null:
return order(_that);case WireAccountEvent_Reconnected() when reconnected != null:
return reconnected(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( WireBalance field0)?  balance,TResult Function( WireOrder field0)?  order,TResult Function()?  reconnected,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireAccountEvent_Balance() when balance != null:
return balance(_that.field0);case WireAccountEvent_Order() when order != null:
return order(_that.field0);case WireAccountEvent_Reconnected() when reconnected != null:
return reconnected();case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( WireBalance field0)  balance,required TResult Function( WireOrder field0)  order,required TResult Function()  reconnected,}) {final _that = this;
switch (_that) {
case WireAccountEvent_Balance():
return balance(_that.field0);case WireAccountEvent_Order():
return order(_that.field0);case WireAccountEvent_Reconnected():
return reconnected();}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( WireBalance field0)?  balance,TResult? Function( WireOrder field0)?  order,TResult? Function()?  reconnected,}) {final _that = this;
switch (_that) {
case WireAccountEvent_Balance() when balance != null:
return balance(_that.field0);case WireAccountEvent_Order() when order != null:
return order(_that.field0);case WireAccountEvent_Reconnected() when reconnected != null:
return reconnected();case _:
  return null;

}
}

}

/// @nodoc


class WireAccountEvent_Balance extends WireAccountEvent {
  const WireAccountEvent_Balance(this.field0): super._();


 final  WireBalance field0;

/// Create a copy of WireAccountEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireAccountEvent_BalanceCopyWith<WireAccountEvent_Balance> get copyWith => _$WireAccountEvent_BalanceCopyWithImpl<WireAccountEvent_Balance>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireAccountEvent_Balance&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireAccountEvent.balance(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireAccountEvent_BalanceCopyWith<$Res> implements $WireAccountEventCopyWith<$Res> {
  factory $WireAccountEvent_BalanceCopyWith(WireAccountEvent_Balance value, $Res Function(WireAccountEvent_Balance) _then) = _$WireAccountEvent_BalanceCopyWithImpl;
@useResult
$Res call({
 WireBalance field0
});




}
/// @nodoc
class _$WireAccountEvent_BalanceCopyWithImpl<$Res>
    implements $WireAccountEvent_BalanceCopyWith<$Res> {
  _$WireAccountEvent_BalanceCopyWithImpl(this._self, this._then);

  final WireAccountEvent_Balance _self;
  final $Res Function(WireAccountEvent_Balance) _then;

/// Create a copy of WireAccountEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireAccountEvent_Balance(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireBalance,
  ));
}


}

/// @nodoc


class WireAccountEvent_Order extends WireAccountEvent {
  const WireAccountEvent_Order(this.field0): super._();


 final  WireOrder field0;

/// Create a copy of WireAccountEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireAccountEvent_OrderCopyWith<WireAccountEvent_Order> get copyWith => _$WireAccountEvent_OrderCopyWithImpl<WireAccountEvent_Order>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireAccountEvent_Order&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireAccountEvent.order(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireAccountEvent_OrderCopyWith<$Res> implements $WireAccountEventCopyWith<$Res> {
  factory $WireAccountEvent_OrderCopyWith(WireAccountEvent_Order value, $Res Function(WireAccountEvent_Order) _then) = _$WireAccountEvent_OrderCopyWithImpl;
@useResult
$Res call({
 WireOrder field0
});




}
/// @nodoc
class _$WireAccountEvent_OrderCopyWithImpl<$Res>
    implements $WireAccountEvent_OrderCopyWith<$Res> {
  _$WireAccountEvent_OrderCopyWithImpl(this._self, this._then);

  final WireAccountEvent_Order _self;
  final $Res Function(WireAccountEvent_Order) _then;

/// Create a copy of WireAccountEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireAccountEvent_Order(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireOrder,
  ));
}


}

/// @nodoc


class WireAccountEvent_Reconnected extends WireAccountEvent {
  const WireAccountEvent_Reconnected(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireAccountEvent_Reconnected);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireAccountEvent.reconnected()';
}


}




/// @nodoc
mixin _$WireAccountStreamItem {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireAccountStreamItem);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireAccountStreamItem()';
}


}

/// @nodoc
class $WireAccountStreamItemCopyWith<$Res>  {
$WireAccountStreamItemCopyWith(WireAccountStreamItem _, $Res Function(WireAccountStreamItem) __);
}


/// Adds pattern-matching-related methods to [WireAccountStreamItem].
extension WireAccountStreamItemPatterns on WireAccountStreamItem {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireAccountStreamItem_Event value)?  event,TResult Function( WireAccountStreamItem_Error value)?  error,TResult Function( WireAccountStreamItem_End value)?  end,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireAccountStreamItem_Event() when event != null:
return event(_that);case WireAccountStreamItem_Error() when error != null:
return error(_that);case WireAccountStreamItem_End() when end != null:
return end(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireAccountStreamItem_Event value)  event,required TResult Function( WireAccountStreamItem_Error value)  error,required TResult Function( WireAccountStreamItem_End value)  end,}){
final _that = this;
switch (_that) {
case WireAccountStreamItem_Event():
return event(_that);case WireAccountStreamItem_Error():
return error(_that);case WireAccountStreamItem_End():
return end(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireAccountStreamItem_Event value)?  event,TResult? Function( WireAccountStreamItem_Error value)?  error,TResult? Function( WireAccountStreamItem_End value)?  end,}){
final _that = this;
switch (_that) {
case WireAccountStreamItem_Event() when event != null:
return event(_that);case WireAccountStreamItem_Error() when error != null:
return error(_that);case WireAccountStreamItem_End() when end != null:
return end(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( WireAccountEvent field0)?  event,TResult Function( NativeError field0)?  error,TResult Function()?  end,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireAccountStreamItem_Event() when event != null:
return event(_that.field0);case WireAccountStreamItem_Error() when error != null:
return error(_that.field0);case WireAccountStreamItem_End() when end != null:
return end();case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( WireAccountEvent field0)  event,required TResult Function( NativeError field0)  error,required TResult Function()  end,}) {final _that = this;
switch (_that) {
case WireAccountStreamItem_Event():
return event(_that.field0);case WireAccountStreamItem_Error():
return error(_that.field0);case WireAccountStreamItem_End():
return end();}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( WireAccountEvent field0)?  event,TResult? Function( NativeError field0)?  error,TResult? Function()?  end,}) {final _that = this;
switch (_that) {
case WireAccountStreamItem_Event() when event != null:
return event(_that.field0);case WireAccountStreamItem_Error() when error != null:
return error(_that.field0);case WireAccountStreamItem_End() when end != null:
return end();case _:
  return null;

}
}

}

/// @nodoc


class WireAccountStreamItem_Event extends WireAccountStreamItem {
  const WireAccountStreamItem_Event(this.field0): super._();


 final  WireAccountEvent field0;

/// Create a copy of WireAccountStreamItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireAccountStreamItem_EventCopyWith<WireAccountStreamItem_Event> get copyWith => _$WireAccountStreamItem_EventCopyWithImpl<WireAccountStreamItem_Event>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireAccountStreamItem_Event&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireAccountStreamItem.event(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireAccountStreamItem_EventCopyWith<$Res> implements $WireAccountStreamItemCopyWith<$Res> {
  factory $WireAccountStreamItem_EventCopyWith(WireAccountStreamItem_Event value, $Res Function(WireAccountStreamItem_Event) _then) = _$WireAccountStreamItem_EventCopyWithImpl;
@useResult
$Res call({
 WireAccountEvent field0
});


$WireAccountEventCopyWith<$Res> get field0;

}
/// @nodoc
class _$WireAccountStreamItem_EventCopyWithImpl<$Res>
    implements $WireAccountStreamItem_EventCopyWith<$Res> {
  _$WireAccountStreamItem_EventCopyWithImpl(this._self, this._then);

  final WireAccountStreamItem_Event _self;
  final $Res Function(WireAccountStreamItem_Event) _then;

/// Create a copy of WireAccountStreamItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireAccountStreamItem_Event(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireAccountEvent,
  ));
}

/// Create a copy of WireAccountStreamItem
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$WireAccountEventCopyWith<$Res> get field0 {

  return $WireAccountEventCopyWith<$Res>(_self.field0, (value) {
    return _then(_self.copyWith(field0: value));
  });
}
}

/// @nodoc


class WireAccountStreamItem_Error extends WireAccountStreamItem {
  const WireAccountStreamItem_Error(this.field0): super._();


 final  NativeError field0;

/// Create a copy of WireAccountStreamItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireAccountStreamItem_ErrorCopyWith<WireAccountStreamItem_Error> get copyWith => _$WireAccountStreamItem_ErrorCopyWithImpl<WireAccountStreamItem_Error>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireAccountStreamItem_Error&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireAccountStreamItem.error(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireAccountStreamItem_ErrorCopyWith<$Res> implements $WireAccountStreamItemCopyWith<$Res> {
  factory $WireAccountStreamItem_ErrorCopyWith(WireAccountStreamItem_Error value, $Res Function(WireAccountStreamItem_Error) _then) = _$WireAccountStreamItem_ErrorCopyWithImpl;
@useResult
$Res call({
 NativeError field0
});




}
/// @nodoc
class _$WireAccountStreamItem_ErrorCopyWithImpl<$Res>
    implements $WireAccountStreamItem_ErrorCopyWith<$Res> {
  _$WireAccountStreamItem_ErrorCopyWithImpl(this._self, this._then);

  final WireAccountStreamItem_Error _self;
  final $Res Function(WireAccountStreamItem_Error) _then;

/// Create a copy of WireAccountStreamItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireAccountStreamItem_Error(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as NativeError,
  ));
}


}

/// @nodoc


class WireAccountStreamItem_End extends WireAccountStreamItem {
  const WireAccountStreamItem_End(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireAccountStreamItem_End);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireAccountStreamItem.end()';
}


}




/// @nodoc
mixin _$WireBinanceAccountStreamItem {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBinanceAccountStreamItem);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireBinanceAccountStreamItem()';
}


}

/// @nodoc
class $WireBinanceAccountStreamItemCopyWith<$Res>  {
$WireBinanceAccountStreamItemCopyWith(WireBinanceAccountStreamItem _, $Res Function(WireBinanceAccountStreamItem) __);
}


/// Adds pattern-matching-related methods to [WireBinanceAccountStreamItem].
extension WireBinanceAccountStreamItemPatterns on WireBinanceAccountStreamItem {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireBinanceAccountStreamItem_Event value)?  event,TResult Function( WireBinanceAccountStreamItem_Error value)?  error,TResult Function( WireBinanceAccountStreamItem_End value)?  end,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireBinanceAccountStreamItem_Event() when event != null:
return event(_that);case WireBinanceAccountStreamItem_Error() when error != null:
return error(_that);case WireBinanceAccountStreamItem_End() when end != null:
return end(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireBinanceAccountStreamItem_Event value)  event,required TResult Function( WireBinanceAccountStreamItem_Error value)  error,required TResult Function( WireBinanceAccountStreamItem_End value)  end,}){
final _that = this;
switch (_that) {
case WireBinanceAccountStreamItem_Event():
return event(_that);case WireBinanceAccountStreamItem_Error():
return error(_that);case WireBinanceAccountStreamItem_End():
return end(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireBinanceAccountStreamItem_Event value)?  event,TResult? Function( WireBinanceAccountStreamItem_Error value)?  error,TResult? Function( WireBinanceAccountStreamItem_End value)?  end,}){
final _that = this;
switch (_that) {
case WireBinanceAccountStreamItem_Event() when event != null:
return event(_that);case WireBinanceAccountStreamItem_Error() when error != null:
return error(_that);case WireBinanceAccountStreamItem_End() when end != null:
return end(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( WireBinanceAccountStreamEvent field0)?  event,TResult Function( NativeError field0)?  error,TResult Function()?  end,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireBinanceAccountStreamItem_Event() when event != null:
return event(_that.field0);case WireBinanceAccountStreamItem_Error() when error != null:
return error(_that.field0);case WireBinanceAccountStreamItem_End() when end != null:
return end();case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( WireBinanceAccountStreamEvent field0)  event,required TResult Function( NativeError field0)  error,required TResult Function()  end,}) {final _that = this;
switch (_that) {
case WireBinanceAccountStreamItem_Event():
return event(_that.field0);case WireBinanceAccountStreamItem_Error():
return error(_that.field0);case WireBinanceAccountStreamItem_End():
return end();}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( WireBinanceAccountStreamEvent field0)?  event,TResult? Function( NativeError field0)?  error,TResult? Function()?  end,}) {final _that = this;
switch (_that) {
case WireBinanceAccountStreamItem_Event() when event != null:
return event(_that.field0);case WireBinanceAccountStreamItem_Error() when error != null:
return error(_that.field0);case WireBinanceAccountStreamItem_End() when end != null:
return end();case _:
  return null;

}
}

}

/// @nodoc


class WireBinanceAccountStreamItem_Event extends WireBinanceAccountStreamItem {
  const WireBinanceAccountStreamItem_Event(this.field0): super._();


 final  WireBinanceAccountStreamEvent field0;

/// Create a copy of WireBinanceAccountStreamItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireBinanceAccountStreamItem_EventCopyWith<WireBinanceAccountStreamItem_Event> get copyWith => _$WireBinanceAccountStreamItem_EventCopyWithImpl<WireBinanceAccountStreamItem_Event>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBinanceAccountStreamItem_Event&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireBinanceAccountStreamItem.event(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireBinanceAccountStreamItem_EventCopyWith<$Res> implements $WireBinanceAccountStreamItemCopyWith<$Res> {
  factory $WireBinanceAccountStreamItem_EventCopyWith(WireBinanceAccountStreamItem_Event value, $Res Function(WireBinanceAccountStreamItem_Event) _then) = _$WireBinanceAccountStreamItem_EventCopyWithImpl;
@useResult
$Res call({
 WireBinanceAccountStreamEvent field0
});


$WireBinanceAccountStreamEventCopyWith<$Res> get field0;

}
/// @nodoc
class _$WireBinanceAccountStreamItem_EventCopyWithImpl<$Res>
    implements $WireBinanceAccountStreamItem_EventCopyWith<$Res> {
  _$WireBinanceAccountStreamItem_EventCopyWithImpl(this._self, this._then);

  final WireBinanceAccountStreamItem_Event _self;
  final $Res Function(WireBinanceAccountStreamItem_Event) _then;

/// Create a copy of WireBinanceAccountStreamItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireBinanceAccountStreamItem_Event(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireBinanceAccountStreamEvent,
  ));
}

/// Create a copy of WireBinanceAccountStreamItem
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$WireBinanceAccountStreamEventCopyWith<$Res> get field0 {

  return $WireBinanceAccountStreamEventCopyWith<$Res>(_self.field0, (value) {
    return _then(_self.copyWith(field0: value));
  });
}
}

/// @nodoc


class WireBinanceAccountStreamItem_Error extends WireBinanceAccountStreamItem {
  const WireBinanceAccountStreamItem_Error(this.field0): super._();


 final  NativeError field0;

/// Create a copy of WireBinanceAccountStreamItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireBinanceAccountStreamItem_ErrorCopyWith<WireBinanceAccountStreamItem_Error> get copyWith => _$WireBinanceAccountStreamItem_ErrorCopyWithImpl<WireBinanceAccountStreamItem_Error>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBinanceAccountStreamItem_Error&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireBinanceAccountStreamItem.error(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireBinanceAccountStreamItem_ErrorCopyWith<$Res> implements $WireBinanceAccountStreamItemCopyWith<$Res> {
  factory $WireBinanceAccountStreamItem_ErrorCopyWith(WireBinanceAccountStreamItem_Error value, $Res Function(WireBinanceAccountStreamItem_Error) _then) = _$WireBinanceAccountStreamItem_ErrorCopyWithImpl;
@useResult
$Res call({
 NativeError field0
});




}
/// @nodoc
class _$WireBinanceAccountStreamItem_ErrorCopyWithImpl<$Res>
    implements $WireBinanceAccountStreamItem_ErrorCopyWith<$Res> {
  _$WireBinanceAccountStreamItem_ErrorCopyWithImpl(this._self, this._then);

  final WireBinanceAccountStreamItem_Error _self;
  final $Res Function(WireBinanceAccountStreamItem_Error) _then;

/// Create a copy of WireBinanceAccountStreamItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireBinanceAccountStreamItem_Error(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as NativeError,
  ));
}


}

/// @nodoc


class WireBinanceAccountStreamItem_End extends WireBinanceAccountStreamItem {
  const WireBinanceAccountStreamItem_End(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBinanceAccountStreamItem_End);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireBinanceAccountStreamItem.end()';
}


}




/// @nodoc
mixin _$WireBinanceMarketStreamItem {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBinanceMarketStreamItem);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireBinanceMarketStreamItem()';
}


}

/// @nodoc
class $WireBinanceMarketStreamItemCopyWith<$Res>  {
$WireBinanceMarketStreamItemCopyWith(WireBinanceMarketStreamItem _, $Res Function(WireBinanceMarketStreamItem) __);
}


/// Adds pattern-matching-related methods to [WireBinanceMarketStreamItem].
extension WireBinanceMarketStreamItemPatterns on WireBinanceMarketStreamItem {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireBinanceMarketStreamItem_Event value)?  event,TResult Function( WireBinanceMarketStreamItem_Error value)?  error,TResult Function( WireBinanceMarketStreamItem_End value)?  end,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireBinanceMarketStreamItem_Event() when event != null:
return event(_that);case WireBinanceMarketStreamItem_Error() when error != null:
return error(_that);case WireBinanceMarketStreamItem_End() when end != null:
return end(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireBinanceMarketStreamItem_Event value)  event,required TResult Function( WireBinanceMarketStreamItem_Error value)  error,required TResult Function( WireBinanceMarketStreamItem_End value)  end,}){
final _that = this;
switch (_that) {
case WireBinanceMarketStreamItem_Event():
return event(_that);case WireBinanceMarketStreamItem_Error():
return error(_that);case WireBinanceMarketStreamItem_End():
return end(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireBinanceMarketStreamItem_Event value)?  event,TResult? Function( WireBinanceMarketStreamItem_Error value)?  error,TResult? Function( WireBinanceMarketStreamItem_End value)?  end,}){
final _that = this;
switch (_that) {
case WireBinanceMarketStreamItem_Event() when event != null:
return event(_that);case WireBinanceMarketStreamItem_Error() when error != null:
return error(_that);case WireBinanceMarketStreamItem_End() when end != null:
return end(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( WireBinanceMarketEvent field0)?  event,TResult Function( NativeError field0)?  error,TResult Function()?  end,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireBinanceMarketStreamItem_Event() when event != null:
return event(_that.field0);case WireBinanceMarketStreamItem_Error() when error != null:
return error(_that.field0);case WireBinanceMarketStreamItem_End() when end != null:
return end();case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( WireBinanceMarketEvent field0)  event,required TResult Function( NativeError field0)  error,required TResult Function()  end,}) {final _that = this;
switch (_that) {
case WireBinanceMarketStreamItem_Event():
return event(_that.field0);case WireBinanceMarketStreamItem_Error():
return error(_that.field0);case WireBinanceMarketStreamItem_End():
return end();}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( WireBinanceMarketEvent field0)?  event,TResult? Function( NativeError field0)?  error,TResult? Function()?  end,}) {final _that = this;
switch (_that) {
case WireBinanceMarketStreamItem_Event() when event != null:
return event(_that.field0);case WireBinanceMarketStreamItem_Error() when error != null:
return error(_that.field0);case WireBinanceMarketStreamItem_End() when end != null:
return end();case _:
  return null;

}
}

}

/// @nodoc


class WireBinanceMarketStreamItem_Event extends WireBinanceMarketStreamItem {
  const WireBinanceMarketStreamItem_Event(this.field0): super._();


 final  WireBinanceMarketEvent field0;

/// Create a copy of WireBinanceMarketStreamItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireBinanceMarketStreamItem_EventCopyWith<WireBinanceMarketStreamItem_Event> get copyWith => _$WireBinanceMarketStreamItem_EventCopyWithImpl<WireBinanceMarketStreamItem_Event>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBinanceMarketStreamItem_Event&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireBinanceMarketStreamItem.event(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireBinanceMarketStreamItem_EventCopyWith<$Res> implements $WireBinanceMarketStreamItemCopyWith<$Res> {
  factory $WireBinanceMarketStreamItem_EventCopyWith(WireBinanceMarketStreamItem_Event value, $Res Function(WireBinanceMarketStreamItem_Event) _then) = _$WireBinanceMarketStreamItem_EventCopyWithImpl;
@useResult
$Res call({
 WireBinanceMarketEvent field0
});


$WireBinanceMarketEventCopyWith<$Res> get field0;

}
/// @nodoc
class _$WireBinanceMarketStreamItem_EventCopyWithImpl<$Res>
    implements $WireBinanceMarketStreamItem_EventCopyWith<$Res> {
  _$WireBinanceMarketStreamItem_EventCopyWithImpl(this._self, this._then);

  final WireBinanceMarketStreamItem_Event _self;
  final $Res Function(WireBinanceMarketStreamItem_Event) _then;

/// Create a copy of WireBinanceMarketStreamItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireBinanceMarketStreamItem_Event(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireBinanceMarketEvent,
  ));
}

/// Create a copy of WireBinanceMarketStreamItem
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$WireBinanceMarketEventCopyWith<$Res> get field0 {

  return $WireBinanceMarketEventCopyWith<$Res>(_self.field0, (value) {
    return _then(_self.copyWith(field0: value));
  });
}
}

/// @nodoc


class WireBinanceMarketStreamItem_Error extends WireBinanceMarketStreamItem {
  const WireBinanceMarketStreamItem_Error(this.field0): super._();


 final  NativeError field0;

/// Create a copy of WireBinanceMarketStreamItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireBinanceMarketStreamItem_ErrorCopyWith<WireBinanceMarketStreamItem_Error> get copyWith => _$WireBinanceMarketStreamItem_ErrorCopyWithImpl<WireBinanceMarketStreamItem_Error>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBinanceMarketStreamItem_Error&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireBinanceMarketStreamItem.error(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireBinanceMarketStreamItem_ErrorCopyWith<$Res> implements $WireBinanceMarketStreamItemCopyWith<$Res> {
  factory $WireBinanceMarketStreamItem_ErrorCopyWith(WireBinanceMarketStreamItem_Error value, $Res Function(WireBinanceMarketStreamItem_Error) _then) = _$WireBinanceMarketStreamItem_ErrorCopyWithImpl;
@useResult
$Res call({
 NativeError field0
});




}
/// @nodoc
class _$WireBinanceMarketStreamItem_ErrorCopyWithImpl<$Res>
    implements $WireBinanceMarketStreamItem_ErrorCopyWith<$Res> {
  _$WireBinanceMarketStreamItem_ErrorCopyWithImpl(this._self, this._then);

  final WireBinanceMarketStreamItem_Error _self;
  final $Res Function(WireBinanceMarketStreamItem_Error) _then;

/// Create a copy of WireBinanceMarketStreamItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireBinanceMarketStreamItem_Error(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as NativeError,
  ));
}


}

/// @nodoc


class WireBinanceMarketStreamItem_End extends WireBinanceMarketStreamItem {
  const WireBinanceMarketStreamItem_End(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBinanceMarketStreamItem_End);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireBinanceMarketStreamItem.end()';
}


}




/// @nodoc
mixin _$WireBithumbAccountStreamItem {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBithumbAccountStreamItem);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireBithumbAccountStreamItem()';
}


}

/// @nodoc
class $WireBithumbAccountStreamItemCopyWith<$Res>  {
$WireBithumbAccountStreamItemCopyWith(WireBithumbAccountStreamItem _, $Res Function(WireBithumbAccountStreamItem) __);
}


/// Adds pattern-matching-related methods to [WireBithumbAccountStreamItem].
extension WireBithumbAccountStreamItemPatterns on WireBithumbAccountStreamItem {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireBithumbAccountStreamItem_Event value)?  event,TResult Function( WireBithumbAccountStreamItem_Error value)?  error,TResult Function( WireBithumbAccountStreamItem_End value)?  end,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireBithumbAccountStreamItem_Event() when event != null:
return event(_that);case WireBithumbAccountStreamItem_Error() when error != null:
return error(_that);case WireBithumbAccountStreamItem_End() when end != null:
return end(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireBithumbAccountStreamItem_Event value)  event,required TResult Function( WireBithumbAccountStreamItem_Error value)  error,required TResult Function( WireBithumbAccountStreamItem_End value)  end,}){
final _that = this;
switch (_that) {
case WireBithumbAccountStreamItem_Event():
return event(_that);case WireBithumbAccountStreamItem_Error():
return error(_that);case WireBithumbAccountStreamItem_End():
return end(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireBithumbAccountStreamItem_Event value)?  event,TResult? Function( WireBithumbAccountStreamItem_Error value)?  error,TResult? Function( WireBithumbAccountStreamItem_End value)?  end,}){
final _that = this;
switch (_that) {
case WireBithumbAccountStreamItem_Event() when event != null:
return event(_that);case WireBithumbAccountStreamItem_Error() when error != null:
return error(_that);case WireBithumbAccountStreamItem_End() when end != null:
return end(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( WireBithumbAccountEvent field0)?  event,TResult Function( NativeError field0)?  error,TResult Function()?  end,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireBithumbAccountStreamItem_Event() when event != null:
return event(_that.field0);case WireBithumbAccountStreamItem_Error() when error != null:
return error(_that.field0);case WireBithumbAccountStreamItem_End() when end != null:
return end();case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( WireBithumbAccountEvent field0)  event,required TResult Function( NativeError field0)  error,required TResult Function()  end,}) {final _that = this;
switch (_that) {
case WireBithumbAccountStreamItem_Event():
return event(_that.field0);case WireBithumbAccountStreamItem_Error():
return error(_that.field0);case WireBithumbAccountStreamItem_End():
return end();}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( WireBithumbAccountEvent field0)?  event,TResult? Function( NativeError field0)?  error,TResult? Function()?  end,}) {final _that = this;
switch (_that) {
case WireBithumbAccountStreamItem_Event() when event != null:
return event(_that.field0);case WireBithumbAccountStreamItem_Error() when error != null:
return error(_that.field0);case WireBithumbAccountStreamItem_End() when end != null:
return end();case _:
  return null;

}
}

}

/// @nodoc


class WireBithumbAccountStreamItem_Event extends WireBithumbAccountStreamItem {
  const WireBithumbAccountStreamItem_Event(this.field0): super._();


 final  WireBithumbAccountEvent field0;

/// Create a copy of WireBithumbAccountStreamItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireBithumbAccountStreamItem_EventCopyWith<WireBithumbAccountStreamItem_Event> get copyWith => _$WireBithumbAccountStreamItem_EventCopyWithImpl<WireBithumbAccountStreamItem_Event>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBithumbAccountStreamItem_Event&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireBithumbAccountStreamItem.event(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireBithumbAccountStreamItem_EventCopyWith<$Res> implements $WireBithumbAccountStreamItemCopyWith<$Res> {
  factory $WireBithumbAccountStreamItem_EventCopyWith(WireBithumbAccountStreamItem_Event value, $Res Function(WireBithumbAccountStreamItem_Event) _then) = _$WireBithumbAccountStreamItem_EventCopyWithImpl;
@useResult
$Res call({
 WireBithumbAccountEvent field0
});


$WireBithumbAccountEventCopyWith<$Res> get field0;

}
/// @nodoc
class _$WireBithumbAccountStreamItem_EventCopyWithImpl<$Res>
    implements $WireBithumbAccountStreamItem_EventCopyWith<$Res> {
  _$WireBithumbAccountStreamItem_EventCopyWithImpl(this._self, this._then);

  final WireBithumbAccountStreamItem_Event _self;
  final $Res Function(WireBithumbAccountStreamItem_Event) _then;

/// Create a copy of WireBithumbAccountStreamItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireBithumbAccountStreamItem_Event(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireBithumbAccountEvent,
  ));
}

/// Create a copy of WireBithumbAccountStreamItem
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$WireBithumbAccountEventCopyWith<$Res> get field0 {

  return $WireBithumbAccountEventCopyWith<$Res>(_self.field0, (value) {
    return _then(_self.copyWith(field0: value));
  });
}
}

/// @nodoc


class WireBithumbAccountStreamItem_Error extends WireBithumbAccountStreamItem {
  const WireBithumbAccountStreamItem_Error(this.field0): super._();


 final  NativeError field0;

/// Create a copy of WireBithumbAccountStreamItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireBithumbAccountStreamItem_ErrorCopyWith<WireBithumbAccountStreamItem_Error> get copyWith => _$WireBithumbAccountStreamItem_ErrorCopyWithImpl<WireBithumbAccountStreamItem_Error>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBithumbAccountStreamItem_Error&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireBithumbAccountStreamItem.error(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireBithumbAccountStreamItem_ErrorCopyWith<$Res> implements $WireBithumbAccountStreamItemCopyWith<$Res> {
  factory $WireBithumbAccountStreamItem_ErrorCopyWith(WireBithumbAccountStreamItem_Error value, $Res Function(WireBithumbAccountStreamItem_Error) _then) = _$WireBithumbAccountStreamItem_ErrorCopyWithImpl;
@useResult
$Res call({
 NativeError field0
});




}
/// @nodoc
class _$WireBithumbAccountStreamItem_ErrorCopyWithImpl<$Res>
    implements $WireBithumbAccountStreamItem_ErrorCopyWith<$Res> {
  _$WireBithumbAccountStreamItem_ErrorCopyWithImpl(this._self, this._then);

  final WireBithumbAccountStreamItem_Error _self;
  final $Res Function(WireBithumbAccountStreamItem_Error) _then;

/// Create a copy of WireBithumbAccountStreamItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireBithumbAccountStreamItem_Error(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as NativeError,
  ));
}


}

/// @nodoc


class WireBithumbAccountStreamItem_End extends WireBithumbAccountStreamItem {
  const WireBithumbAccountStreamItem_End(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBithumbAccountStreamItem_End);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireBithumbAccountStreamItem.end()';
}


}




/// @nodoc
mixin _$WireBithumbMarketStreamItem {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBithumbMarketStreamItem);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireBithumbMarketStreamItem()';
}


}

/// @nodoc
class $WireBithumbMarketStreamItemCopyWith<$Res>  {
$WireBithumbMarketStreamItemCopyWith(WireBithumbMarketStreamItem _, $Res Function(WireBithumbMarketStreamItem) __);
}


/// Adds pattern-matching-related methods to [WireBithumbMarketStreamItem].
extension WireBithumbMarketStreamItemPatterns on WireBithumbMarketStreamItem {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireBithumbMarketStreamItem_Event value)?  event,TResult Function( WireBithumbMarketStreamItem_Error value)?  error,TResult Function( WireBithumbMarketStreamItem_End value)?  end,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireBithumbMarketStreamItem_Event() when event != null:
return event(_that);case WireBithumbMarketStreamItem_Error() when error != null:
return error(_that);case WireBithumbMarketStreamItem_End() when end != null:
return end(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireBithumbMarketStreamItem_Event value)  event,required TResult Function( WireBithumbMarketStreamItem_Error value)  error,required TResult Function( WireBithumbMarketStreamItem_End value)  end,}){
final _that = this;
switch (_that) {
case WireBithumbMarketStreamItem_Event():
return event(_that);case WireBithumbMarketStreamItem_Error():
return error(_that);case WireBithumbMarketStreamItem_End():
return end(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireBithumbMarketStreamItem_Event value)?  event,TResult? Function( WireBithumbMarketStreamItem_Error value)?  error,TResult? Function( WireBithumbMarketStreamItem_End value)?  end,}){
final _that = this;
switch (_that) {
case WireBithumbMarketStreamItem_Event() when event != null:
return event(_that);case WireBithumbMarketStreamItem_Error() when error != null:
return error(_that);case WireBithumbMarketStreamItem_End() when end != null:
return end(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( WireBithumbMarketEvent field0)?  event,TResult Function( NativeError field0)?  error,TResult Function()?  end,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireBithumbMarketStreamItem_Event() when event != null:
return event(_that.field0);case WireBithumbMarketStreamItem_Error() when error != null:
return error(_that.field0);case WireBithumbMarketStreamItem_End() when end != null:
return end();case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( WireBithumbMarketEvent field0)  event,required TResult Function( NativeError field0)  error,required TResult Function()  end,}) {final _that = this;
switch (_that) {
case WireBithumbMarketStreamItem_Event():
return event(_that.field0);case WireBithumbMarketStreamItem_Error():
return error(_that.field0);case WireBithumbMarketStreamItem_End():
return end();}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( WireBithumbMarketEvent field0)?  event,TResult? Function( NativeError field0)?  error,TResult? Function()?  end,}) {final _that = this;
switch (_that) {
case WireBithumbMarketStreamItem_Event() when event != null:
return event(_that.field0);case WireBithumbMarketStreamItem_Error() when error != null:
return error(_that.field0);case WireBithumbMarketStreamItem_End() when end != null:
return end();case _:
  return null;

}
}

}

/// @nodoc


class WireBithumbMarketStreamItem_Event extends WireBithumbMarketStreamItem {
  const WireBithumbMarketStreamItem_Event(this.field0): super._();


 final  WireBithumbMarketEvent field0;

/// Create a copy of WireBithumbMarketStreamItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireBithumbMarketStreamItem_EventCopyWith<WireBithumbMarketStreamItem_Event> get copyWith => _$WireBithumbMarketStreamItem_EventCopyWithImpl<WireBithumbMarketStreamItem_Event>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBithumbMarketStreamItem_Event&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireBithumbMarketStreamItem.event(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireBithumbMarketStreamItem_EventCopyWith<$Res> implements $WireBithumbMarketStreamItemCopyWith<$Res> {
  factory $WireBithumbMarketStreamItem_EventCopyWith(WireBithumbMarketStreamItem_Event value, $Res Function(WireBithumbMarketStreamItem_Event) _then) = _$WireBithumbMarketStreamItem_EventCopyWithImpl;
@useResult
$Res call({
 WireBithumbMarketEvent field0
});


$WireBithumbMarketEventCopyWith<$Res> get field0;

}
/// @nodoc
class _$WireBithumbMarketStreamItem_EventCopyWithImpl<$Res>
    implements $WireBithumbMarketStreamItem_EventCopyWith<$Res> {
  _$WireBithumbMarketStreamItem_EventCopyWithImpl(this._self, this._then);

  final WireBithumbMarketStreamItem_Event _self;
  final $Res Function(WireBithumbMarketStreamItem_Event) _then;

/// Create a copy of WireBithumbMarketStreamItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireBithumbMarketStreamItem_Event(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireBithumbMarketEvent,
  ));
}

/// Create a copy of WireBithumbMarketStreamItem
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$WireBithumbMarketEventCopyWith<$Res> get field0 {

  return $WireBithumbMarketEventCopyWith<$Res>(_self.field0, (value) {
    return _then(_self.copyWith(field0: value));
  });
}
}

/// @nodoc


class WireBithumbMarketStreamItem_Error extends WireBithumbMarketStreamItem {
  const WireBithumbMarketStreamItem_Error(this.field0): super._();


 final  NativeError field0;

/// Create a copy of WireBithumbMarketStreamItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireBithumbMarketStreamItem_ErrorCopyWith<WireBithumbMarketStreamItem_Error> get copyWith => _$WireBithumbMarketStreamItem_ErrorCopyWithImpl<WireBithumbMarketStreamItem_Error>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBithumbMarketStreamItem_Error&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireBithumbMarketStreamItem.error(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireBithumbMarketStreamItem_ErrorCopyWith<$Res> implements $WireBithumbMarketStreamItemCopyWith<$Res> {
  factory $WireBithumbMarketStreamItem_ErrorCopyWith(WireBithumbMarketStreamItem_Error value, $Res Function(WireBithumbMarketStreamItem_Error) _then) = _$WireBithumbMarketStreamItem_ErrorCopyWithImpl;
@useResult
$Res call({
 NativeError field0
});




}
/// @nodoc
class _$WireBithumbMarketStreamItem_ErrorCopyWithImpl<$Res>
    implements $WireBithumbMarketStreamItem_ErrorCopyWith<$Res> {
  _$WireBithumbMarketStreamItem_ErrorCopyWithImpl(this._self, this._then);

  final WireBithumbMarketStreamItem_Error _self;
  final $Res Function(WireBithumbMarketStreamItem_Error) _then;

/// Create a copy of WireBithumbMarketStreamItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireBithumbMarketStreamItem_Error(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as NativeError,
  ));
}


}

/// @nodoc


class WireBithumbMarketStreamItem_End extends WireBithumbMarketStreamItem {
  const WireBithumbMarketStreamItem_End(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBithumbMarketStreamItem_End);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireBithumbMarketStreamItem.end()';
}


}




/// @nodoc
mixin _$WireHyperliquidAccountStreamItem {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidAccountStreamItem);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireHyperliquidAccountStreamItem()';
}


}

/// @nodoc
class $WireHyperliquidAccountStreamItemCopyWith<$Res>  {
$WireHyperliquidAccountStreamItemCopyWith(WireHyperliquidAccountStreamItem _, $Res Function(WireHyperliquidAccountStreamItem) __);
}


/// Adds pattern-matching-related methods to [WireHyperliquidAccountStreamItem].
extension WireHyperliquidAccountStreamItemPatterns on WireHyperliquidAccountStreamItem {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireHyperliquidAccountStreamItem_Event value)?  event,TResult Function( WireHyperliquidAccountStreamItem_Error value)?  error,TResult Function( WireHyperliquidAccountStreamItem_End value)?  end,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireHyperliquidAccountStreamItem_Event() when event != null:
return event(_that);case WireHyperliquidAccountStreamItem_Error() when error != null:
return error(_that);case WireHyperliquidAccountStreamItem_End() when end != null:
return end(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireHyperliquidAccountStreamItem_Event value)  event,required TResult Function( WireHyperliquidAccountStreamItem_Error value)  error,required TResult Function( WireHyperliquidAccountStreamItem_End value)  end,}){
final _that = this;
switch (_that) {
case WireHyperliquidAccountStreamItem_Event():
return event(_that);case WireHyperliquidAccountStreamItem_Error():
return error(_that);case WireHyperliquidAccountStreamItem_End():
return end(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireHyperliquidAccountStreamItem_Event value)?  event,TResult? Function( WireHyperliquidAccountStreamItem_Error value)?  error,TResult? Function( WireHyperliquidAccountStreamItem_End value)?  end,}){
final _that = this;
switch (_that) {
case WireHyperliquidAccountStreamItem_Event() when event != null:
return event(_that);case WireHyperliquidAccountStreamItem_Error() when error != null:
return error(_that);case WireHyperliquidAccountStreamItem_End() when end != null:
return end(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( WireHyperliquidAccountEvent field0)?  event,TResult Function( NativeError field0)?  error,TResult Function()?  end,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireHyperliquidAccountStreamItem_Event() when event != null:
return event(_that.field0);case WireHyperliquidAccountStreamItem_Error() when error != null:
return error(_that.field0);case WireHyperliquidAccountStreamItem_End() when end != null:
return end();case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( WireHyperliquidAccountEvent field0)  event,required TResult Function( NativeError field0)  error,required TResult Function()  end,}) {final _that = this;
switch (_that) {
case WireHyperliquidAccountStreamItem_Event():
return event(_that.field0);case WireHyperliquidAccountStreamItem_Error():
return error(_that.field0);case WireHyperliquidAccountStreamItem_End():
return end();}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( WireHyperliquidAccountEvent field0)?  event,TResult? Function( NativeError field0)?  error,TResult? Function()?  end,}) {final _that = this;
switch (_that) {
case WireHyperliquidAccountStreamItem_Event() when event != null:
return event(_that.field0);case WireHyperliquidAccountStreamItem_Error() when error != null:
return error(_that.field0);case WireHyperliquidAccountStreamItem_End() when end != null:
return end();case _:
  return null;

}
}

}

/// @nodoc


class WireHyperliquidAccountStreamItem_Event extends WireHyperliquidAccountStreamItem {
  const WireHyperliquidAccountStreamItem_Event(this.field0): super._();


 final  WireHyperliquidAccountEvent field0;

/// Create a copy of WireHyperliquidAccountStreamItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireHyperliquidAccountStreamItem_EventCopyWith<WireHyperliquidAccountStreamItem_Event> get copyWith => _$WireHyperliquidAccountStreamItem_EventCopyWithImpl<WireHyperliquidAccountStreamItem_Event>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidAccountStreamItem_Event&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireHyperliquidAccountStreamItem.event(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireHyperliquidAccountStreamItem_EventCopyWith<$Res> implements $WireHyperliquidAccountStreamItemCopyWith<$Res> {
  factory $WireHyperliquidAccountStreamItem_EventCopyWith(WireHyperliquidAccountStreamItem_Event value, $Res Function(WireHyperliquidAccountStreamItem_Event) _then) = _$WireHyperliquidAccountStreamItem_EventCopyWithImpl;
@useResult
$Res call({
 WireHyperliquidAccountEvent field0
});


$WireHyperliquidAccountEventCopyWith<$Res> get field0;

}
/// @nodoc
class _$WireHyperliquidAccountStreamItem_EventCopyWithImpl<$Res>
    implements $WireHyperliquidAccountStreamItem_EventCopyWith<$Res> {
  _$WireHyperliquidAccountStreamItem_EventCopyWithImpl(this._self, this._then);

  final WireHyperliquidAccountStreamItem_Event _self;
  final $Res Function(WireHyperliquidAccountStreamItem_Event) _then;

/// Create a copy of WireHyperliquidAccountStreamItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireHyperliquidAccountStreamItem_Event(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireHyperliquidAccountEvent,
  ));
}

/// Create a copy of WireHyperliquidAccountStreamItem
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$WireHyperliquidAccountEventCopyWith<$Res> get field0 {

  return $WireHyperliquidAccountEventCopyWith<$Res>(_self.field0, (value) {
    return _then(_self.copyWith(field0: value));
  });
}
}

/// @nodoc


class WireHyperliquidAccountStreamItem_Error extends WireHyperliquidAccountStreamItem {
  const WireHyperliquidAccountStreamItem_Error(this.field0): super._();


 final  NativeError field0;

/// Create a copy of WireHyperliquidAccountStreamItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireHyperliquidAccountStreamItem_ErrorCopyWith<WireHyperliquidAccountStreamItem_Error> get copyWith => _$WireHyperliquidAccountStreamItem_ErrorCopyWithImpl<WireHyperliquidAccountStreamItem_Error>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidAccountStreamItem_Error&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireHyperliquidAccountStreamItem.error(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireHyperliquidAccountStreamItem_ErrorCopyWith<$Res> implements $WireHyperliquidAccountStreamItemCopyWith<$Res> {
  factory $WireHyperliquidAccountStreamItem_ErrorCopyWith(WireHyperliquidAccountStreamItem_Error value, $Res Function(WireHyperliquidAccountStreamItem_Error) _then) = _$WireHyperliquidAccountStreamItem_ErrorCopyWithImpl;
@useResult
$Res call({
 NativeError field0
});




}
/// @nodoc
class _$WireHyperliquidAccountStreamItem_ErrorCopyWithImpl<$Res>
    implements $WireHyperliquidAccountStreamItem_ErrorCopyWith<$Res> {
  _$WireHyperliquidAccountStreamItem_ErrorCopyWithImpl(this._self, this._then);

  final WireHyperliquidAccountStreamItem_Error _self;
  final $Res Function(WireHyperliquidAccountStreamItem_Error) _then;

/// Create a copy of WireHyperliquidAccountStreamItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireHyperliquidAccountStreamItem_Error(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as NativeError,
  ));
}


}

/// @nodoc


class WireHyperliquidAccountStreamItem_End extends WireHyperliquidAccountStreamItem {
  const WireHyperliquidAccountStreamItem_End(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidAccountStreamItem_End);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireHyperliquidAccountStreamItem.end()';
}


}




/// @nodoc
mixin _$WireHyperliquidMarketStreamItem {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidMarketStreamItem);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireHyperliquidMarketStreamItem()';
}


}

/// @nodoc
class $WireHyperliquidMarketStreamItemCopyWith<$Res>  {
$WireHyperliquidMarketStreamItemCopyWith(WireHyperliquidMarketStreamItem _, $Res Function(WireHyperliquidMarketStreamItem) __);
}


/// Adds pattern-matching-related methods to [WireHyperliquidMarketStreamItem].
extension WireHyperliquidMarketStreamItemPatterns on WireHyperliquidMarketStreamItem {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireHyperliquidMarketStreamItem_Event value)?  event,TResult Function( WireHyperliquidMarketStreamItem_Error value)?  error,TResult Function( WireHyperliquidMarketStreamItem_End value)?  end,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireHyperliquidMarketStreamItem_Event() when event != null:
return event(_that);case WireHyperliquidMarketStreamItem_Error() when error != null:
return error(_that);case WireHyperliquidMarketStreamItem_End() when end != null:
return end(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireHyperliquidMarketStreamItem_Event value)  event,required TResult Function( WireHyperliquidMarketStreamItem_Error value)  error,required TResult Function( WireHyperliquidMarketStreamItem_End value)  end,}){
final _that = this;
switch (_that) {
case WireHyperliquidMarketStreamItem_Event():
return event(_that);case WireHyperliquidMarketStreamItem_Error():
return error(_that);case WireHyperliquidMarketStreamItem_End():
return end(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireHyperliquidMarketStreamItem_Event value)?  event,TResult? Function( WireHyperliquidMarketStreamItem_Error value)?  error,TResult? Function( WireHyperliquidMarketStreamItem_End value)?  end,}){
final _that = this;
switch (_that) {
case WireHyperliquidMarketStreamItem_Event() when event != null:
return event(_that);case WireHyperliquidMarketStreamItem_Error() when error != null:
return error(_that);case WireHyperliquidMarketStreamItem_End() when end != null:
return end(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( WireHyperliquidMarketEvent field0)?  event,TResult Function( NativeError field0)?  error,TResult Function()?  end,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireHyperliquidMarketStreamItem_Event() when event != null:
return event(_that.field0);case WireHyperliquidMarketStreamItem_Error() when error != null:
return error(_that.field0);case WireHyperliquidMarketStreamItem_End() when end != null:
return end();case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( WireHyperliquidMarketEvent field0)  event,required TResult Function( NativeError field0)  error,required TResult Function()  end,}) {final _that = this;
switch (_that) {
case WireHyperliquidMarketStreamItem_Event():
return event(_that.field0);case WireHyperliquidMarketStreamItem_Error():
return error(_that.field0);case WireHyperliquidMarketStreamItem_End():
return end();}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( WireHyperliquidMarketEvent field0)?  event,TResult? Function( NativeError field0)?  error,TResult? Function()?  end,}) {final _that = this;
switch (_that) {
case WireHyperliquidMarketStreamItem_Event() when event != null:
return event(_that.field0);case WireHyperliquidMarketStreamItem_Error() when error != null:
return error(_that.field0);case WireHyperliquidMarketStreamItem_End() when end != null:
return end();case _:
  return null;

}
}

}

/// @nodoc


class WireHyperliquidMarketStreamItem_Event extends WireHyperliquidMarketStreamItem {
  const WireHyperliquidMarketStreamItem_Event(this.field0): super._();


 final  WireHyperliquidMarketEvent field0;

/// Create a copy of WireHyperliquidMarketStreamItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireHyperliquidMarketStreamItem_EventCopyWith<WireHyperliquidMarketStreamItem_Event> get copyWith => _$WireHyperliquidMarketStreamItem_EventCopyWithImpl<WireHyperliquidMarketStreamItem_Event>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidMarketStreamItem_Event&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireHyperliquidMarketStreamItem.event(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireHyperliquidMarketStreamItem_EventCopyWith<$Res> implements $WireHyperliquidMarketStreamItemCopyWith<$Res> {
  factory $WireHyperliquidMarketStreamItem_EventCopyWith(WireHyperliquidMarketStreamItem_Event value, $Res Function(WireHyperliquidMarketStreamItem_Event) _then) = _$WireHyperliquidMarketStreamItem_EventCopyWithImpl;
@useResult
$Res call({
 WireHyperliquidMarketEvent field0
});


$WireHyperliquidMarketEventCopyWith<$Res> get field0;

}
/// @nodoc
class _$WireHyperliquidMarketStreamItem_EventCopyWithImpl<$Res>
    implements $WireHyperliquidMarketStreamItem_EventCopyWith<$Res> {
  _$WireHyperliquidMarketStreamItem_EventCopyWithImpl(this._self, this._then);

  final WireHyperliquidMarketStreamItem_Event _self;
  final $Res Function(WireHyperliquidMarketStreamItem_Event) _then;

/// Create a copy of WireHyperliquidMarketStreamItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireHyperliquidMarketStreamItem_Event(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireHyperliquidMarketEvent,
  ));
}

/// Create a copy of WireHyperliquidMarketStreamItem
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$WireHyperliquidMarketEventCopyWith<$Res> get field0 {

  return $WireHyperliquidMarketEventCopyWith<$Res>(_self.field0, (value) {
    return _then(_self.copyWith(field0: value));
  });
}
}

/// @nodoc


class WireHyperliquidMarketStreamItem_Error extends WireHyperliquidMarketStreamItem {
  const WireHyperliquidMarketStreamItem_Error(this.field0): super._();


 final  NativeError field0;

/// Create a copy of WireHyperliquidMarketStreamItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireHyperliquidMarketStreamItem_ErrorCopyWith<WireHyperliquidMarketStreamItem_Error> get copyWith => _$WireHyperliquidMarketStreamItem_ErrorCopyWithImpl<WireHyperliquidMarketStreamItem_Error>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidMarketStreamItem_Error&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireHyperliquidMarketStreamItem.error(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireHyperliquidMarketStreamItem_ErrorCopyWith<$Res> implements $WireHyperliquidMarketStreamItemCopyWith<$Res> {
  factory $WireHyperliquidMarketStreamItem_ErrorCopyWith(WireHyperliquidMarketStreamItem_Error value, $Res Function(WireHyperliquidMarketStreamItem_Error) _then) = _$WireHyperliquidMarketStreamItem_ErrorCopyWithImpl;
@useResult
$Res call({
 NativeError field0
});




}
/// @nodoc
class _$WireHyperliquidMarketStreamItem_ErrorCopyWithImpl<$Res>
    implements $WireHyperliquidMarketStreamItem_ErrorCopyWith<$Res> {
  _$WireHyperliquidMarketStreamItem_ErrorCopyWithImpl(this._self, this._then);

  final WireHyperliquidMarketStreamItem_Error _self;
  final $Res Function(WireHyperliquidMarketStreamItem_Error) _then;

/// Create a copy of WireHyperliquidMarketStreamItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireHyperliquidMarketStreamItem_Error(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as NativeError,
  ));
}


}

/// @nodoc


class WireHyperliquidMarketStreamItem_End extends WireHyperliquidMarketStreamItem {
  const WireHyperliquidMarketStreamItem_End(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidMarketStreamItem_End);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireHyperliquidMarketStreamItem.end()';
}


}




/// @nodoc
mixin _$WireMarketEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireMarketEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireMarketEvent()';
}


}

/// @nodoc
class $WireMarketEventCopyWith<$Res>  {
$WireMarketEventCopyWith(WireMarketEvent _, $Res Function(WireMarketEvent) __);
}


/// Adds pattern-matching-related methods to [WireMarketEvent].
extension WireMarketEventPatterns on WireMarketEvent {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireMarketEvent_Trade value)?  trade,TResult Function( WireMarketEvent_OrderBook value)?  orderBook,TResult Function( WireMarketEvent_Ticker value)?  ticker,TResult Function( WireMarketEvent_Candle value)?  candle,TResult Function( WireMarketEvent_Reconnected value)?  reconnected,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireMarketEvent_Trade() when trade != null:
return trade(_that);case WireMarketEvent_OrderBook() when orderBook != null:
return orderBook(_that);case WireMarketEvent_Ticker() when ticker != null:
return ticker(_that);case WireMarketEvent_Candle() when candle != null:
return candle(_that);case WireMarketEvent_Reconnected() when reconnected != null:
return reconnected(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireMarketEvent_Trade value)  trade,required TResult Function( WireMarketEvent_OrderBook value)  orderBook,required TResult Function( WireMarketEvent_Ticker value)  ticker,required TResult Function( WireMarketEvent_Candle value)  candle,required TResult Function( WireMarketEvent_Reconnected value)  reconnected,}){
final _that = this;
switch (_that) {
case WireMarketEvent_Trade():
return trade(_that);case WireMarketEvent_OrderBook():
return orderBook(_that);case WireMarketEvent_Ticker():
return ticker(_that);case WireMarketEvent_Candle():
return candle(_that);case WireMarketEvent_Reconnected():
return reconnected(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireMarketEvent_Trade value)?  trade,TResult? Function( WireMarketEvent_OrderBook value)?  orderBook,TResult? Function( WireMarketEvent_Ticker value)?  ticker,TResult? Function( WireMarketEvent_Candle value)?  candle,TResult? Function( WireMarketEvent_Reconnected value)?  reconnected,}){
final _that = this;
switch (_that) {
case WireMarketEvent_Trade() when trade != null:
return trade(_that);case WireMarketEvent_OrderBook() when orderBook != null:
return orderBook(_that);case WireMarketEvent_Ticker() when ticker != null:
return ticker(_that);case WireMarketEvent_Candle() when candle != null:
return candle(_that);case WireMarketEvent_Reconnected() when reconnected != null:
return reconnected(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( WireTrade field0)?  trade,TResult Function( WireOrderBook field0)?  orderBook,TResult Function( WireTicker field0)?  ticker,TResult Function( WireCandle field0)?  candle,TResult Function()?  reconnected,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireMarketEvent_Trade() when trade != null:
return trade(_that.field0);case WireMarketEvent_OrderBook() when orderBook != null:
return orderBook(_that.field0);case WireMarketEvent_Ticker() when ticker != null:
return ticker(_that.field0);case WireMarketEvent_Candle() when candle != null:
return candle(_that.field0);case WireMarketEvent_Reconnected() when reconnected != null:
return reconnected();case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( WireTrade field0)  trade,required TResult Function( WireOrderBook field0)  orderBook,required TResult Function( WireTicker field0)  ticker,required TResult Function( WireCandle field0)  candle,required TResult Function()  reconnected,}) {final _that = this;
switch (_that) {
case WireMarketEvent_Trade():
return trade(_that.field0);case WireMarketEvent_OrderBook():
return orderBook(_that.field0);case WireMarketEvent_Ticker():
return ticker(_that.field0);case WireMarketEvent_Candle():
return candle(_that.field0);case WireMarketEvent_Reconnected():
return reconnected();}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( WireTrade field0)?  trade,TResult? Function( WireOrderBook field0)?  orderBook,TResult? Function( WireTicker field0)?  ticker,TResult? Function( WireCandle field0)?  candle,TResult? Function()?  reconnected,}) {final _that = this;
switch (_that) {
case WireMarketEvent_Trade() when trade != null:
return trade(_that.field0);case WireMarketEvent_OrderBook() when orderBook != null:
return orderBook(_that.field0);case WireMarketEvent_Ticker() when ticker != null:
return ticker(_that.field0);case WireMarketEvent_Candle() when candle != null:
return candle(_that.field0);case WireMarketEvent_Reconnected() when reconnected != null:
return reconnected();case _:
  return null;

}
}

}

/// @nodoc


class WireMarketEvent_Trade extends WireMarketEvent {
  const WireMarketEvent_Trade(this.field0): super._();


 final  WireTrade field0;

/// Create a copy of WireMarketEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireMarketEvent_TradeCopyWith<WireMarketEvent_Trade> get copyWith => _$WireMarketEvent_TradeCopyWithImpl<WireMarketEvent_Trade>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireMarketEvent_Trade&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireMarketEvent.trade(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireMarketEvent_TradeCopyWith<$Res> implements $WireMarketEventCopyWith<$Res> {
  factory $WireMarketEvent_TradeCopyWith(WireMarketEvent_Trade value, $Res Function(WireMarketEvent_Trade) _then) = _$WireMarketEvent_TradeCopyWithImpl;
@useResult
$Res call({
 WireTrade field0
});




}
/// @nodoc
class _$WireMarketEvent_TradeCopyWithImpl<$Res>
    implements $WireMarketEvent_TradeCopyWith<$Res> {
  _$WireMarketEvent_TradeCopyWithImpl(this._self, this._then);

  final WireMarketEvent_Trade _self;
  final $Res Function(WireMarketEvent_Trade) _then;

/// Create a copy of WireMarketEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireMarketEvent_Trade(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireTrade,
  ));
}


}

/// @nodoc


class WireMarketEvent_OrderBook extends WireMarketEvent {
  const WireMarketEvent_OrderBook(this.field0): super._();


 final  WireOrderBook field0;

/// Create a copy of WireMarketEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireMarketEvent_OrderBookCopyWith<WireMarketEvent_OrderBook> get copyWith => _$WireMarketEvent_OrderBookCopyWithImpl<WireMarketEvent_OrderBook>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireMarketEvent_OrderBook&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireMarketEvent.orderBook(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireMarketEvent_OrderBookCopyWith<$Res> implements $WireMarketEventCopyWith<$Res> {
  factory $WireMarketEvent_OrderBookCopyWith(WireMarketEvent_OrderBook value, $Res Function(WireMarketEvent_OrderBook) _then) = _$WireMarketEvent_OrderBookCopyWithImpl;
@useResult
$Res call({
 WireOrderBook field0
});




}
/// @nodoc
class _$WireMarketEvent_OrderBookCopyWithImpl<$Res>
    implements $WireMarketEvent_OrderBookCopyWith<$Res> {
  _$WireMarketEvent_OrderBookCopyWithImpl(this._self, this._then);

  final WireMarketEvent_OrderBook _self;
  final $Res Function(WireMarketEvent_OrderBook) _then;

/// Create a copy of WireMarketEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireMarketEvent_OrderBook(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireOrderBook,
  ));
}


}

/// @nodoc


class WireMarketEvent_Ticker extends WireMarketEvent {
  const WireMarketEvent_Ticker(this.field0): super._();


 final  WireTicker field0;

/// Create a copy of WireMarketEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireMarketEvent_TickerCopyWith<WireMarketEvent_Ticker> get copyWith => _$WireMarketEvent_TickerCopyWithImpl<WireMarketEvent_Ticker>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireMarketEvent_Ticker&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireMarketEvent.ticker(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireMarketEvent_TickerCopyWith<$Res> implements $WireMarketEventCopyWith<$Res> {
  factory $WireMarketEvent_TickerCopyWith(WireMarketEvent_Ticker value, $Res Function(WireMarketEvent_Ticker) _then) = _$WireMarketEvent_TickerCopyWithImpl;
@useResult
$Res call({
 WireTicker field0
});




}
/// @nodoc
class _$WireMarketEvent_TickerCopyWithImpl<$Res>
    implements $WireMarketEvent_TickerCopyWith<$Res> {
  _$WireMarketEvent_TickerCopyWithImpl(this._self, this._then);

  final WireMarketEvent_Ticker _self;
  final $Res Function(WireMarketEvent_Ticker) _then;

/// Create a copy of WireMarketEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireMarketEvent_Ticker(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireTicker,
  ));
}


}

/// @nodoc


class WireMarketEvent_Candle extends WireMarketEvent {
  const WireMarketEvent_Candle(this.field0): super._();


 final  WireCandle field0;

/// Create a copy of WireMarketEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireMarketEvent_CandleCopyWith<WireMarketEvent_Candle> get copyWith => _$WireMarketEvent_CandleCopyWithImpl<WireMarketEvent_Candle>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireMarketEvent_Candle&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireMarketEvent.candle(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireMarketEvent_CandleCopyWith<$Res> implements $WireMarketEventCopyWith<$Res> {
  factory $WireMarketEvent_CandleCopyWith(WireMarketEvent_Candle value, $Res Function(WireMarketEvent_Candle) _then) = _$WireMarketEvent_CandleCopyWithImpl;
@useResult
$Res call({
 WireCandle field0
});




}
/// @nodoc
class _$WireMarketEvent_CandleCopyWithImpl<$Res>
    implements $WireMarketEvent_CandleCopyWith<$Res> {
  _$WireMarketEvent_CandleCopyWithImpl(this._self, this._then);

  final WireMarketEvent_Candle _self;
  final $Res Function(WireMarketEvent_Candle) _then;

/// Create a copy of WireMarketEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireMarketEvent_Candle(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireCandle,
  ));
}


}

/// @nodoc


class WireMarketEvent_Reconnected extends WireMarketEvent {
  const WireMarketEvent_Reconnected(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireMarketEvent_Reconnected);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireMarketEvent.reconnected()';
}


}




/// @nodoc
mixin _$WireMarketStreamItem {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireMarketStreamItem);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireMarketStreamItem()';
}


}

/// @nodoc
class $WireMarketStreamItemCopyWith<$Res>  {
$WireMarketStreamItemCopyWith(WireMarketStreamItem _, $Res Function(WireMarketStreamItem) __);
}


/// Adds pattern-matching-related methods to [WireMarketStreamItem].
extension WireMarketStreamItemPatterns on WireMarketStreamItem {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireMarketStreamItem_Event value)?  event,TResult Function( WireMarketStreamItem_Error value)?  error,TResult Function( WireMarketStreamItem_End value)?  end,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireMarketStreamItem_Event() when event != null:
return event(_that);case WireMarketStreamItem_Error() when error != null:
return error(_that);case WireMarketStreamItem_End() when end != null:
return end(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireMarketStreamItem_Event value)  event,required TResult Function( WireMarketStreamItem_Error value)  error,required TResult Function( WireMarketStreamItem_End value)  end,}){
final _that = this;
switch (_that) {
case WireMarketStreamItem_Event():
return event(_that);case WireMarketStreamItem_Error():
return error(_that);case WireMarketStreamItem_End():
return end(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireMarketStreamItem_Event value)?  event,TResult? Function( WireMarketStreamItem_Error value)?  error,TResult? Function( WireMarketStreamItem_End value)?  end,}){
final _that = this;
switch (_that) {
case WireMarketStreamItem_Event() when event != null:
return event(_that);case WireMarketStreamItem_Error() when error != null:
return error(_that);case WireMarketStreamItem_End() when end != null:
return end(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( WireMarketEvent field0)?  event,TResult Function( NativeError field0)?  error,TResult Function()?  end,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireMarketStreamItem_Event() when event != null:
return event(_that.field0);case WireMarketStreamItem_Error() when error != null:
return error(_that.field0);case WireMarketStreamItem_End() when end != null:
return end();case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( WireMarketEvent field0)  event,required TResult Function( NativeError field0)  error,required TResult Function()  end,}) {final _that = this;
switch (_that) {
case WireMarketStreamItem_Event():
return event(_that.field0);case WireMarketStreamItem_Error():
return error(_that.field0);case WireMarketStreamItem_End():
return end();}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( WireMarketEvent field0)?  event,TResult? Function( NativeError field0)?  error,TResult? Function()?  end,}) {final _that = this;
switch (_that) {
case WireMarketStreamItem_Event() when event != null:
return event(_that.field0);case WireMarketStreamItem_Error() when error != null:
return error(_that.field0);case WireMarketStreamItem_End() when end != null:
return end();case _:
  return null;

}
}

}

/// @nodoc


class WireMarketStreamItem_Event extends WireMarketStreamItem {
  const WireMarketStreamItem_Event(this.field0): super._();


 final  WireMarketEvent field0;

/// Create a copy of WireMarketStreamItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireMarketStreamItem_EventCopyWith<WireMarketStreamItem_Event> get copyWith => _$WireMarketStreamItem_EventCopyWithImpl<WireMarketStreamItem_Event>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireMarketStreamItem_Event&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireMarketStreamItem.event(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireMarketStreamItem_EventCopyWith<$Res> implements $WireMarketStreamItemCopyWith<$Res> {
  factory $WireMarketStreamItem_EventCopyWith(WireMarketStreamItem_Event value, $Res Function(WireMarketStreamItem_Event) _then) = _$WireMarketStreamItem_EventCopyWithImpl;
@useResult
$Res call({
 WireMarketEvent field0
});


$WireMarketEventCopyWith<$Res> get field0;

}
/// @nodoc
class _$WireMarketStreamItem_EventCopyWithImpl<$Res>
    implements $WireMarketStreamItem_EventCopyWith<$Res> {
  _$WireMarketStreamItem_EventCopyWithImpl(this._self, this._then);

  final WireMarketStreamItem_Event _self;
  final $Res Function(WireMarketStreamItem_Event) _then;

/// Create a copy of WireMarketStreamItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireMarketStreamItem_Event(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireMarketEvent,
  ));
}

/// Create a copy of WireMarketStreamItem
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$WireMarketEventCopyWith<$Res> get field0 {

  return $WireMarketEventCopyWith<$Res>(_self.field0, (value) {
    return _then(_self.copyWith(field0: value));
  });
}
}

/// @nodoc


class WireMarketStreamItem_Error extends WireMarketStreamItem {
  const WireMarketStreamItem_Error(this.field0): super._();


 final  NativeError field0;

/// Create a copy of WireMarketStreamItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireMarketStreamItem_ErrorCopyWith<WireMarketStreamItem_Error> get copyWith => _$WireMarketStreamItem_ErrorCopyWithImpl<WireMarketStreamItem_Error>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireMarketStreamItem_Error&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireMarketStreamItem.error(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireMarketStreamItem_ErrorCopyWith<$Res> implements $WireMarketStreamItemCopyWith<$Res> {
  factory $WireMarketStreamItem_ErrorCopyWith(WireMarketStreamItem_Error value, $Res Function(WireMarketStreamItem_Error) _then) = _$WireMarketStreamItem_ErrorCopyWithImpl;
@useResult
$Res call({
 NativeError field0
});




}
/// @nodoc
class _$WireMarketStreamItem_ErrorCopyWithImpl<$Res>
    implements $WireMarketStreamItem_ErrorCopyWith<$Res> {
  _$WireMarketStreamItem_ErrorCopyWithImpl(this._self, this._then);

  final WireMarketStreamItem_Error _self;
  final $Res Function(WireMarketStreamItem_Error) _then;

/// Create a copy of WireMarketStreamItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireMarketStreamItem_Error(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as NativeError,
  ));
}


}

/// @nodoc


class WireMarketStreamItem_End extends WireMarketStreamItem {
  const WireMarketStreamItem_End(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireMarketStreamItem_End);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireMarketStreamItem.end()';
}


}




/// @nodoc
mixin _$WireUpbitAccountStreamItem {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitAccountStreamItem);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireUpbitAccountStreamItem()';
}


}

/// @nodoc
class $WireUpbitAccountStreamItemCopyWith<$Res>  {
$WireUpbitAccountStreamItemCopyWith(WireUpbitAccountStreamItem _, $Res Function(WireUpbitAccountStreamItem) __);
}


/// Adds pattern-matching-related methods to [WireUpbitAccountStreamItem].
extension WireUpbitAccountStreamItemPatterns on WireUpbitAccountStreamItem {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireUpbitAccountStreamItem_Event value)?  event,TResult Function( WireUpbitAccountStreamItem_Error value)?  error,TResult Function( WireUpbitAccountStreamItem_End value)?  end,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireUpbitAccountStreamItem_Event() when event != null:
return event(_that);case WireUpbitAccountStreamItem_Error() when error != null:
return error(_that);case WireUpbitAccountStreamItem_End() when end != null:
return end(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireUpbitAccountStreamItem_Event value)  event,required TResult Function( WireUpbitAccountStreamItem_Error value)  error,required TResult Function( WireUpbitAccountStreamItem_End value)  end,}){
final _that = this;
switch (_that) {
case WireUpbitAccountStreamItem_Event():
return event(_that);case WireUpbitAccountStreamItem_Error():
return error(_that);case WireUpbitAccountStreamItem_End():
return end(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireUpbitAccountStreamItem_Event value)?  event,TResult? Function( WireUpbitAccountStreamItem_Error value)?  error,TResult? Function( WireUpbitAccountStreamItem_End value)?  end,}){
final _that = this;
switch (_that) {
case WireUpbitAccountStreamItem_Event() when event != null:
return event(_that);case WireUpbitAccountStreamItem_Error() when error != null:
return error(_that);case WireUpbitAccountStreamItem_End() when end != null:
return end(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( WireUpbitAccountStreamEvent field0)?  event,TResult Function( NativeError field0)?  error,TResult Function()?  end,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireUpbitAccountStreamItem_Event() when event != null:
return event(_that.field0);case WireUpbitAccountStreamItem_Error() when error != null:
return error(_that.field0);case WireUpbitAccountStreamItem_End() when end != null:
return end();case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( WireUpbitAccountStreamEvent field0)  event,required TResult Function( NativeError field0)  error,required TResult Function()  end,}) {final _that = this;
switch (_that) {
case WireUpbitAccountStreamItem_Event():
return event(_that.field0);case WireUpbitAccountStreamItem_Error():
return error(_that.field0);case WireUpbitAccountStreamItem_End():
return end();}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( WireUpbitAccountStreamEvent field0)?  event,TResult? Function( NativeError field0)?  error,TResult? Function()?  end,}) {final _that = this;
switch (_that) {
case WireUpbitAccountStreamItem_Event() when event != null:
return event(_that.field0);case WireUpbitAccountStreamItem_Error() when error != null:
return error(_that.field0);case WireUpbitAccountStreamItem_End() when end != null:
return end();case _:
  return null;

}
}

}

/// @nodoc


class WireUpbitAccountStreamItem_Event extends WireUpbitAccountStreamItem {
  const WireUpbitAccountStreamItem_Event(this.field0): super._();


 final  WireUpbitAccountStreamEvent field0;

/// Create a copy of WireUpbitAccountStreamItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireUpbitAccountStreamItem_EventCopyWith<WireUpbitAccountStreamItem_Event> get copyWith => _$WireUpbitAccountStreamItem_EventCopyWithImpl<WireUpbitAccountStreamItem_Event>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitAccountStreamItem_Event&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireUpbitAccountStreamItem.event(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireUpbitAccountStreamItem_EventCopyWith<$Res> implements $WireUpbitAccountStreamItemCopyWith<$Res> {
  factory $WireUpbitAccountStreamItem_EventCopyWith(WireUpbitAccountStreamItem_Event value, $Res Function(WireUpbitAccountStreamItem_Event) _then) = _$WireUpbitAccountStreamItem_EventCopyWithImpl;
@useResult
$Res call({
 WireUpbitAccountStreamEvent field0
});


$WireUpbitAccountStreamEventCopyWith<$Res> get field0;

}
/// @nodoc
class _$WireUpbitAccountStreamItem_EventCopyWithImpl<$Res>
    implements $WireUpbitAccountStreamItem_EventCopyWith<$Res> {
  _$WireUpbitAccountStreamItem_EventCopyWithImpl(this._self, this._then);

  final WireUpbitAccountStreamItem_Event _self;
  final $Res Function(WireUpbitAccountStreamItem_Event) _then;

/// Create a copy of WireUpbitAccountStreamItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireUpbitAccountStreamItem_Event(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireUpbitAccountStreamEvent,
  ));
}

/// Create a copy of WireUpbitAccountStreamItem
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$WireUpbitAccountStreamEventCopyWith<$Res> get field0 {

  return $WireUpbitAccountStreamEventCopyWith<$Res>(_self.field0, (value) {
    return _then(_self.copyWith(field0: value));
  });
}
}

/// @nodoc


class WireUpbitAccountStreamItem_Error extends WireUpbitAccountStreamItem {
  const WireUpbitAccountStreamItem_Error(this.field0): super._();


 final  NativeError field0;

/// Create a copy of WireUpbitAccountStreamItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireUpbitAccountStreamItem_ErrorCopyWith<WireUpbitAccountStreamItem_Error> get copyWith => _$WireUpbitAccountStreamItem_ErrorCopyWithImpl<WireUpbitAccountStreamItem_Error>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitAccountStreamItem_Error&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireUpbitAccountStreamItem.error(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireUpbitAccountStreamItem_ErrorCopyWith<$Res> implements $WireUpbitAccountStreamItemCopyWith<$Res> {
  factory $WireUpbitAccountStreamItem_ErrorCopyWith(WireUpbitAccountStreamItem_Error value, $Res Function(WireUpbitAccountStreamItem_Error) _then) = _$WireUpbitAccountStreamItem_ErrorCopyWithImpl;
@useResult
$Res call({
 NativeError field0
});




}
/// @nodoc
class _$WireUpbitAccountStreamItem_ErrorCopyWithImpl<$Res>
    implements $WireUpbitAccountStreamItem_ErrorCopyWith<$Res> {
  _$WireUpbitAccountStreamItem_ErrorCopyWithImpl(this._self, this._then);

  final WireUpbitAccountStreamItem_Error _self;
  final $Res Function(WireUpbitAccountStreamItem_Error) _then;

/// Create a copy of WireUpbitAccountStreamItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireUpbitAccountStreamItem_Error(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as NativeError,
  ));
}


}

/// @nodoc


class WireUpbitAccountStreamItem_End extends WireUpbitAccountStreamItem {
  const WireUpbitAccountStreamItem_End(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitAccountStreamItem_End);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireUpbitAccountStreamItem.end()';
}


}




/// @nodoc
mixin _$WireUpbitMarketStreamItem {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitMarketStreamItem);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireUpbitMarketStreamItem()';
}


}

/// @nodoc
class $WireUpbitMarketStreamItemCopyWith<$Res>  {
$WireUpbitMarketStreamItemCopyWith(WireUpbitMarketStreamItem _, $Res Function(WireUpbitMarketStreamItem) __);
}


/// Adds pattern-matching-related methods to [WireUpbitMarketStreamItem].
extension WireUpbitMarketStreamItemPatterns on WireUpbitMarketStreamItem {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireUpbitMarketStreamItem_Event value)?  event,TResult Function( WireUpbitMarketStreamItem_Error value)?  error,TResult Function( WireUpbitMarketStreamItem_End value)?  end,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireUpbitMarketStreamItem_Event() when event != null:
return event(_that);case WireUpbitMarketStreamItem_Error() when error != null:
return error(_that);case WireUpbitMarketStreamItem_End() when end != null:
return end(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireUpbitMarketStreamItem_Event value)  event,required TResult Function( WireUpbitMarketStreamItem_Error value)  error,required TResult Function( WireUpbitMarketStreamItem_End value)  end,}){
final _that = this;
switch (_that) {
case WireUpbitMarketStreamItem_Event():
return event(_that);case WireUpbitMarketStreamItem_Error():
return error(_that);case WireUpbitMarketStreamItem_End():
return end(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireUpbitMarketStreamItem_Event value)?  event,TResult? Function( WireUpbitMarketStreamItem_Error value)?  error,TResult? Function( WireUpbitMarketStreamItem_End value)?  end,}){
final _that = this;
switch (_that) {
case WireUpbitMarketStreamItem_Event() when event != null:
return event(_that);case WireUpbitMarketStreamItem_Error() when error != null:
return error(_that);case WireUpbitMarketStreamItem_End() when end != null:
return end(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( WireUpbitMarketStreamEvent field0)?  event,TResult Function( NativeError field0)?  error,TResult Function()?  end,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireUpbitMarketStreamItem_Event() when event != null:
return event(_that.field0);case WireUpbitMarketStreamItem_Error() when error != null:
return error(_that.field0);case WireUpbitMarketStreamItem_End() when end != null:
return end();case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( WireUpbitMarketStreamEvent field0)  event,required TResult Function( NativeError field0)  error,required TResult Function()  end,}) {final _that = this;
switch (_that) {
case WireUpbitMarketStreamItem_Event():
return event(_that.field0);case WireUpbitMarketStreamItem_Error():
return error(_that.field0);case WireUpbitMarketStreamItem_End():
return end();}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( WireUpbitMarketStreamEvent field0)?  event,TResult? Function( NativeError field0)?  error,TResult? Function()?  end,}) {final _that = this;
switch (_that) {
case WireUpbitMarketStreamItem_Event() when event != null:
return event(_that.field0);case WireUpbitMarketStreamItem_Error() when error != null:
return error(_that.field0);case WireUpbitMarketStreamItem_End() when end != null:
return end();case _:
  return null;

}
}

}

/// @nodoc


class WireUpbitMarketStreamItem_Event extends WireUpbitMarketStreamItem {
  const WireUpbitMarketStreamItem_Event(this.field0): super._();


 final  WireUpbitMarketStreamEvent field0;

/// Create a copy of WireUpbitMarketStreamItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireUpbitMarketStreamItem_EventCopyWith<WireUpbitMarketStreamItem_Event> get copyWith => _$WireUpbitMarketStreamItem_EventCopyWithImpl<WireUpbitMarketStreamItem_Event>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitMarketStreamItem_Event&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireUpbitMarketStreamItem.event(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireUpbitMarketStreamItem_EventCopyWith<$Res> implements $WireUpbitMarketStreamItemCopyWith<$Res> {
  factory $WireUpbitMarketStreamItem_EventCopyWith(WireUpbitMarketStreamItem_Event value, $Res Function(WireUpbitMarketStreamItem_Event) _then) = _$WireUpbitMarketStreamItem_EventCopyWithImpl;
@useResult
$Res call({
 WireUpbitMarketStreamEvent field0
});


$WireUpbitMarketStreamEventCopyWith<$Res> get field0;

}
/// @nodoc
class _$WireUpbitMarketStreamItem_EventCopyWithImpl<$Res>
    implements $WireUpbitMarketStreamItem_EventCopyWith<$Res> {
  _$WireUpbitMarketStreamItem_EventCopyWithImpl(this._self, this._then);

  final WireUpbitMarketStreamItem_Event _self;
  final $Res Function(WireUpbitMarketStreamItem_Event) _then;

/// Create a copy of WireUpbitMarketStreamItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireUpbitMarketStreamItem_Event(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireUpbitMarketStreamEvent,
  ));
}

/// Create a copy of WireUpbitMarketStreamItem
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$WireUpbitMarketStreamEventCopyWith<$Res> get field0 {

  return $WireUpbitMarketStreamEventCopyWith<$Res>(_self.field0, (value) {
    return _then(_self.copyWith(field0: value));
  });
}
}

/// @nodoc


class WireUpbitMarketStreamItem_Error extends WireUpbitMarketStreamItem {
  const WireUpbitMarketStreamItem_Error(this.field0): super._();


 final  NativeError field0;

/// Create a copy of WireUpbitMarketStreamItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireUpbitMarketStreamItem_ErrorCopyWith<WireUpbitMarketStreamItem_Error> get copyWith => _$WireUpbitMarketStreamItem_ErrorCopyWithImpl<WireUpbitMarketStreamItem_Error>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitMarketStreamItem_Error&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireUpbitMarketStreamItem.error(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireUpbitMarketStreamItem_ErrorCopyWith<$Res> implements $WireUpbitMarketStreamItemCopyWith<$Res> {
  factory $WireUpbitMarketStreamItem_ErrorCopyWith(WireUpbitMarketStreamItem_Error value, $Res Function(WireUpbitMarketStreamItem_Error) _then) = _$WireUpbitMarketStreamItem_ErrorCopyWithImpl;
@useResult
$Res call({
 NativeError field0
});




}
/// @nodoc
class _$WireUpbitMarketStreamItem_ErrorCopyWithImpl<$Res>
    implements $WireUpbitMarketStreamItem_ErrorCopyWith<$Res> {
  _$WireUpbitMarketStreamItem_ErrorCopyWithImpl(this._self, this._then);

  final WireUpbitMarketStreamItem_Error _self;
  final $Res Function(WireUpbitMarketStreamItem_Error) _then;

/// Create a copy of WireUpbitMarketStreamItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireUpbitMarketStreamItem_Error(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as NativeError,
  ));
}


}

/// @nodoc


class WireUpbitMarketStreamItem_End extends WireUpbitMarketStreamItem {
  const WireUpbitMarketStreamItem_End(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitMarketStreamItem_End);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireUpbitMarketStreamItem.end()';
}


}




// dart format on

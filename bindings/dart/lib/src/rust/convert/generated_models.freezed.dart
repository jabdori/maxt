// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'generated_models.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$WireBinanceAccountStreamEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBinanceAccountStreamEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireBinanceAccountStreamEvent()';
}


}

/// @nodoc
class $WireBinanceAccountStreamEventCopyWith<$Res>  {
$WireBinanceAccountStreamEventCopyWith(WireBinanceAccountStreamEvent _, $Res Function(WireBinanceAccountStreamEvent) __);
}


/// Adds pattern-matching-related methods to [WireBinanceAccountStreamEvent].
extension WireBinanceAccountStreamEventPatterns on WireBinanceAccountStreamEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireBinanceAccountStreamEvent_Balance value)?  balance,TResult Function( WireBinanceAccountStreamEvent_Order value)?  order,TResult Function( WireBinanceAccountStreamEvent_Other value)?  other,TResult Function( WireBinanceAccountStreamEvent_Reconnected value)?  reconnected,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireBinanceAccountStreamEvent_Balance() when balance != null:
return balance(_that);case WireBinanceAccountStreamEvent_Order() when order != null:
return order(_that);case WireBinanceAccountStreamEvent_Other() when other != null:
return other(_that);case WireBinanceAccountStreamEvent_Reconnected() when reconnected != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireBinanceAccountStreamEvent_Balance value)  balance,required TResult Function( WireBinanceAccountStreamEvent_Order value)  order,required TResult Function( WireBinanceAccountStreamEvent_Other value)  other,required TResult Function( WireBinanceAccountStreamEvent_Reconnected value)  reconnected,}){
final _that = this;
switch (_that) {
case WireBinanceAccountStreamEvent_Balance():
return balance(_that);case WireBinanceAccountStreamEvent_Order():
return order(_that);case WireBinanceAccountStreamEvent_Other():
return other(_that);case WireBinanceAccountStreamEvent_Reconnected():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireBinanceAccountStreamEvent_Balance value)?  balance,TResult? Function( WireBinanceAccountStreamEvent_Order value)?  order,TResult? Function( WireBinanceAccountStreamEvent_Other value)?  other,TResult? Function( WireBinanceAccountStreamEvent_Reconnected value)?  reconnected,}){
final _that = this;
switch (_that) {
case WireBinanceAccountStreamEvent_Balance() when balance != null:
return balance(_that);case WireBinanceAccountStreamEvent_Order() when order != null:
return order(_that);case WireBinanceAccountStreamEvent_Other() when other != null:
return other(_that);case WireBinanceAccountStreamEvent_Reconnected() when reconnected != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( WireBinanceBalanceStreamEvent field0)?  balance,TResult Function( WireBinanceOrderStreamEvent field0)?  order,TResult Function( WireBinanceRawAccountEvent field0)?  other,TResult Function()?  reconnected,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireBinanceAccountStreamEvent_Balance() when balance != null:
return balance(_that.field0);case WireBinanceAccountStreamEvent_Order() when order != null:
return order(_that.field0);case WireBinanceAccountStreamEvent_Other() when other != null:
return other(_that.field0);case WireBinanceAccountStreamEvent_Reconnected() when reconnected != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( WireBinanceBalanceStreamEvent field0)  balance,required TResult Function( WireBinanceOrderStreamEvent field0)  order,required TResult Function( WireBinanceRawAccountEvent field0)  other,required TResult Function()  reconnected,}) {final _that = this;
switch (_that) {
case WireBinanceAccountStreamEvent_Balance():
return balance(_that.field0);case WireBinanceAccountStreamEvent_Order():
return order(_that.field0);case WireBinanceAccountStreamEvent_Other():
return other(_that.field0);case WireBinanceAccountStreamEvent_Reconnected():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( WireBinanceBalanceStreamEvent field0)?  balance,TResult? Function( WireBinanceOrderStreamEvent field0)?  order,TResult? Function( WireBinanceRawAccountEvent field0)?  other,TResult? Function()?  reconnected,}) {final _that = this;
switch (_that) {
case WireBinanceAccountStreamEvent_Balance() when balance != null:
return balance(_that.field0);case WireBinanceAccountStreamEvent_Order() when order != null:
return order(_that.field0);case WireBinanceAccountStreamEvent_Other() when other != null:
return other(_that.field0);case WireBinanceAccountStreamEvent_Reconnected() when reconnected != null:
return reconnected();case _:
  return null;

}
}

}

/// @nodoc


class WireBinanceAccountStreamEvent_Balance extends WireBinanceAccountStreamEvent {
  const WireBinanceAccountStreamEvent_Balance(this.field0): super._();


 final  WireBinanceBalanceStreamEvent field0;

/// Create a copy of WireBinanceAccountStreamEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireBinanceAccountStreamEvent_BalanceCopyWith<WireBinanceAccountStreamEvent_Balance> get copyWith => _$WireBinanceAccountStreamEvent_BalanceCopyWithImpl<WireBinanceAccountStreamEvent_Balance>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBinanceAccountStreamEvent_Balance&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireBinanceAccountStreamEvent.balance(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireBinanceAccountStreamEvent_BalanceCopyWith<$Res> implements $WireBinanceAccountStreamEventCopyWith<$Res> {
  factory $WireBinanceAccountStreamEvent_BalanceCopyWith(WireBinanceAccountStreamEvent_Balance value, $Res Function(WireBinanceAccountStreamEvent_Balance) _then) = _$WireBinanceAccountStreamEvent_BalanceCopyWithImpl;
@useResult
$Res call({
 WireBinanceBalanceStreamEvent field0
});




}
/// @nodoc
class _$WireBinanceAccountStreamEvent_BalanceCopyWithImpl<$Res>
    implements $WireBinanceAccountStreamEvent_BalanceCopyWith<$Res> {
  _$WireBinanceAccountStreamEvent_BalanceCopyWithImpl(this._self, this._then);

  final WireBinanceAccountStreamEvent_Balance _self;
  final $Res Function(WireBinanceAccountStreamEvent_Balance) _then;

/// Create a copy of WireBinanceAccountStreamEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireBinanceAccountStreamEvent_Balance(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireBinanceBalanceStreamEvent,
  ));
}


}

/// @nodoc


class WireBinanceAccountStreamEvent_Order extends WireBinanceAccountStreamEvent {
  const WireBinanceAccountStreamEvent_Order(this.field0): super._();


 final  WireBinanceOrderStreamEvent field0;

/// Create a copy of WireBinanceAccountStreamEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireBinanceAccountStreamEvent_OrderCopyWith<WireBinanceAccountStreamEvent_Order> get copyWith => _$WireBinanceAccountStreamEvent_OrderCopyWithImpl<WireBinanceAccountStreamEvent_Order>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBinanceAccountStreamEvent_Order&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireBinanceAccountStreamEvent.order(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireBinanceAccountStreamEvent_OrderCopyWith<$Res> implements $WireBinanceAccountStreamEventCopyWith<$Res> {
  factory $WireBinanceAccountStreamEvent_OrderCopyWith(WireBinanceAccountStreamEvent_Order value, $Res Function(WireBinanceAccountStreamEvent_Order) _then) = _$WireBinanceAccountStreamEvent_OrderCopyWithImpl;
@useResult
$Res call({
 WireBinanceOrderStreamEvent field0
});




}
/// @nodoc
class _$WireBinanceAccountStreamEvent_OrderCopyWithImpl<$Res>
    implements $WireBinanceAccountStreamEvent_OrderCopyWith<$Res> {
  _$WireBinanceAccountStreamEvent_OrderCopyWithImpl(this._self, this._then);

  final WireBinanceAccountStreamEvent_Order _self;
  final $Res Function(WireBinanceAccountStreamEvent_Order) _then;

/// Create a copy of WireBinanceAccountStreamEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireBinanceAccountStreamEvent_Order(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireBinanceOrderStreamEvent,
  ));
}


}

/// @nodoc


class WireBinanceAccountStreamEvent_Other extends WireBinanceAccountStreamEvent {
  const WireBinanceAccountStreamEvent_Other(this.field0): super._();


 final  WireBinanceRawAccountEvent field0;

/// Create a copy of WireBinanceAccountStreamEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireBinanceAccountStreamEvent_OtherCopyWith<WireBinanceAccountStreamEvent_Other> get copyWith => _$WireBinanceAccountStreamEvent_OtherCopyWithImpl<WireBinanceAccountStreamEvent_Other>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBinanceAccountStreamEvent_Other&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireBinanceAccountStreamEvent.other(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireBinanceAccountStreamEvent_OtherCopyWith<$Res> implements $WireBinanceAccountStreamEventCopyWith<$Res> {
  factory $WireBinanceAccountStreamEvent_OtherCopyWith(WireBinanceAccountStreamEvent_Other value, $Res Function(WireBinanceAccountStreamEvent_Other) _then) = _$WireBinanceAccountStreamEvent_OtherCopyWithImpl;
@useResult
$Res call({
 WireBinanceRawAccountEvent field0
});




}
/// @nodoc
class _$WireBinanceAccountStreamEvent_OtherCopyWithImpl<$Res>
    implements $WireBinanceAccountStreamEvent_OtherCopyWith<$Res> {
  _$WireBinanceAccountStreamEvent_OtherCopyWithImpl(this._self, this._then);

  final WireBinanceAccountStreamEvent_Other _self;
  final $Res Function(WireBinanceAccountStreamEvent_Other) _then;

/// Create a copy of WireBinanceAccountStreamEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireBinanceAccountStreamEvent_Other(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireBinanceRawAccountEvent,
  ));
}


}

/// @nodoc


class WireBinanceAccountStreamEvent_Reconnected extends WireBinanceAccountStreamEvent {
  const WireBinanceAccountStreamEvent_Reconnected(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBinanceAccountStreamEvent_Reconnected);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireBinanceAccountStreamEvent.reconnected()';
}


}




/// @nodoc
mixin _$WireBinanceMarketEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBinanceMarketEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireBinanceMarketEvent()';
}


}

/// @nodoc
class $WireBinanceMarketEventCopyWith<$Res>  {
$WireBinanceMarketEventCopyWith(WireBinanceMarketEvent _, $Res Function(WireBinanceMarketEvent) __);
}


/// Adds pattern-matching-related methods to [WireBinanceMarketEvent].
extension WireBinanceMarketEventPatterns on WireBinanceMarketEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireBinanceMarketEvent_Trade value)?  trade,TResult Function( WireBinanceMarketEvent_OrderBook value)?  orderBook,TResult Function( WireBinanceMarketEvent_Ticker value)?  ticker,TResult Function( WireBinanceMarketEvent_Candle value)?  candle,TResult Function( WireBinanceMarketEvent_Reconnected value)?  reconnected,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireBinanceMarketEvent_Trade() when trade != null:
return trade(_that);case WireBinanceMarketEvent_OrderBook() when orderBook != null:
return orderBook(_that);case WireBinanceMarketEvent_Ticker() when ticker != null:
return ticker(_that);case WireBinanceMarketEvent_Candle() when candle != null:
return candle(_that);case WireBinanceMarketEvent_Reconnected() when reconnected != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireBinanceMarketEvent_Trade value)  trade,required TResult Function( WireBinanceMarketEvent_OrderBook value)  orderBook,required TResult Function( WireBinanceMarketEvent_Ticker value)  ticker,required TResult Function( WireBinanceMarketEvent_Candle value)  candle,required TResult Function( WireBinanceMarketEvent_Reconnected value)  reconnected,}){
final _that = this;
switch (_that) {
case WireBinanceMarketEvent_Trade():
return trade(_that);case WireBinanceMarketEvent_OrderBook():
return orderBook(_that);case WireBinanceMarketEvent_Ticker():
return ticker(_that);case WireBinanceMarketEvent_Candle():
return candle(_that);case WireBinanceMarketEvent_Reconnected():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireBinanceMarketEvent_Trade value)?  trade,TResult? Function( WireBinanceMarketEvent_OrderBook value)?  orderBook,TResult? Function( WireBinanceMarketEvent_Ticker value)?  ticker,TResult? Function( WireBinanceMarketEvent_Candle value)?  candle,TResult? Function( WireBinanceMarketEvent_Reconnected value)?  reconnected,}){
final _that = this;
switch (_that) {
case WireBinanceMarketEvent_Trade() when trade != null:
return trade(_that);case WireBinanceMarketEvent_OrderBook() when orderBook != null:
return orderBook(_that);case WireBinanceMarketEvent_Ticker() when ticker != null:
return ticker(_that);case WireBinanceMarketEvent_Candle() when candle != null:
return candle(_that);case WireBinanceMarketEvent_Reconnected() when reconnected != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( WireBinanceTradeEvent field0)?  trade,TResult Function( WireBinanceOrderBookEvent field0)?  orderBook,TResult Function( WireBinanceTickerEvent field0)?  ticker,TResult Function( WireBinanceCandleEvent field0)?  candle,TResult Function()?  reconnected,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireBinanceMarketEvent_Trade() when trade != null:
return trade(_that.field0);case WireBinanceMarketEvent_OrderBook() when orderBook != null:
return orderBook(_that.field0);case WireBinanceMarketEvent_Ticker() when ticker != null:
return ticker(_that.field0);case WireBinanceMarketEvent_Candle() when candle != null:
return candle(_that.field0);case WireBinanceMarketEvent_Reconnected() when reconnected != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( WireBinanceTradeEvent field0)  trade,required TResult Function( WireBinanceOrderBookEvent field0)  orderBook,required TResult Function( WireBinanceTickerEvent field0)  ticker,required TResult Function( WireBinanceCandleEvent field0)  candle,required TResult Function()  reconnected,}) {final _that = this;
switch (_that) {
case WireBinanceMarketEvent_Trade():
return trade(_that.field0);case WireBinanceMarketEvent_OrderBook():
return orderBook(_that.field0);case WireBinanceMarketEvent_Ticker():
return ticker(_that.field0);case WireBinanceMarketEvent_Candle():
return candle(_that.field0);case WireBinanceMarketEvent_Reconnected():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( WireBinanceTradeEvent field0)?  trade,TResult? Function( WireBinanceOrderBookEvent field0)?  orderBook,TResult? Function( WireBinanceTickerEvent field0)?  ticker,TResult? Function( WireBinanceCandleEvent field0)?  candle,TResult? Function()?  reconnected,}) {final _that = this;
switch (_that) {
case WireBinanceMarketEvent_Trade() when trade != null:
return trade(_that.field0);case WireBinanceMarketEvent_OrderBook() when orderBook != null:
return orderBook(_that.field0);case WireBinanceMarketEvent_Ticker() when ticker != null:
return ticker(_that.field0);case WireBinanceMarketEvent_Candle() when candle != null:
return candle(_that.field0);case WireBinanceMarketEvent_Reconnected() when reconnected != null:
return reconnected();case _:
  return null;

}
}

}

/// @nodoc


class WireBinanceMarketEvent_Trade extends WireBinanceMarketEvent {
  const WireBinanceMarketEvent_Trade(this.field0): super._();


 final  WireBinanceTradeEvent field0;

/// Create a copy of WireBinanceMarketEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireBinanceMarketEvent_TradeCopyWith<WireBinanceMarketEvent_Trade> get copyWith => _$WireBinanceMarketEvent_TradeCopyWithImpl<WireBinanceMarketEvent_Trade>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBinanceMarketEvent_Trade&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireBinanceMarketEvent.trade(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireBinanceMarketEvent_TradeCopyWith<$Res> implements $WireBinanceMarketEventCopyWith<$Res> {
  factory $WireBinanceMarketEvent_TradeCopyWith(WireBinanceMarketEvent_Trade value, $Res Function(WireBinanceMarketEvent_Trade) _then) = _$WireBinanceMarketEvent_TradeCopyWithImpl;
@useResult
$Res call({
 WireBinanceTradeEvent field0
});




}
/// @nodoc
class _$WireBinanceMarketEvent_TradeCopyWithImpl<$Res>
    implements $WireBinanceMarketEvent_TradeCopyWith<$Res> {
  _$WireBinanceMarketEvent_TradeCopyWithImpl(this._self, this._then);

  final WireBinanceMarketEvent_Trade _self;
  final $Res Function(WireBinanceMarketEvent_Trade) _then;

/// Create a copy of WireBinanceMarketEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireBinanceMarketEvent_Trade(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireBinanceTradeEvent,
  ));
}


}

/// @nodoc


class WireBinanceMarketEvent_OrderBook extends WireBinanceMarketEvent {
  const WireBinanceMarketEvent_OrderBook(this.field0): super._();


 final  WireBinanceOrderBookEvent field0;

/// Create a copy of WireBinanceMarketEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireBinanceMarketEvent_OrderBookCopyWith<WireBinanceMarketEvent_OrderBook> get copyWith => _$WireBinanceMarketEvent_OrderBookCopyWithImpl<WireBinanceMarketEvent_OrderBook>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBinanceMarketEvent_OrderBook&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireBinanceMarketEvent.orderBook(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireBinanceMarketEvent_OrderBookCopyWith<$Res> implements $WireBinanceMarketEventCopyWith<$Res> {
  factory $WireBinanceMarketEvent_OrderBookCopyWith(WireBinanceMarketEvent_OrderBook value, $Res Function(WireBinanceMarketEvent_OrderBook) _then) = _$WireBinanceMarketEvent_OrderBookCopyWithImpl;
@useResult
$Res call({
 WireBinanceOrderBookEvent field0
});




}
/// @nodoc
class _$WireBinanceMarketEvent_OrderBookCopyWithImpl<$Res>
    implements $WireBinanceMarketEvent_OrderBookCopyWith<$Res> {
  _$WireBinanceMarketEvent_OrderBookCopyWithImpl(this._self, this._then);

  final WireBinanceMarketEvent_OrderBook _self;
  final $Res Function(WireBinanceMarketEvent_OrderBook) _then;

/// Create a copy of WireBinanceMarketEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireBinanceMarketEvent_OrderBook(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireBinanceOrderBookEvent,
  ));
}


}

/// @nodoc


class WireBinanceMarketEvent_Ticker extends WireBinanceMarketEvent {
  const WireBinanceMarketEvent_Ticker(this.field0): super._();


 final  WireBinanceTickerEvent field0;

/// Create a copy of WireBinanceMarketEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireBinanceMarketEvent_TickerCopyWith<WireBinanceMarketEvent_Ticker> get copyWith => _$WireBinanceMarketEvent_TickerCopyWithImpl<WireBinanceMarketEvent_Ticker>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBinanceMarketEvent_Ticker&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireBinanceMarketEvent.ticker(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireBinanceMarketEvent_TickerCopyWith<$Res> implements $WireBinanceMarketEventCopyWith<$Res> {
  factory $WireBinanceMarketEvent_TickerCopyWith(WireBinanceMarketEvent_Ticker value, $Res Function(WireBinanceMarketEvent_Ticker) _then) = _$WireBinanceMarketEvent_TickerCopyWithImpl;
@useResult
$Res call({
 WireBinanceTickerEvent field0
});




}
/// @nodoc
class _$WireBinanceMarketEvent_TickerCopyWithImpl<$Res>
    implements $WireBinanceMarketEvent_TickerCopyWith<$Res> {
  _$WireBinanceMarketEvent_TickerCopyWithImpl(this._self, this._then);

  final WireBinanceMarketEvent_Ticker _self;
  final $Res Function(WireBinanceMarketEvent_Ticker) _then;

/// Create a copy of WireBinanceMarketEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireBinanceMarketEvent_Ticker(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireBinanceTickerEvent,
  ));
}


}

/// @nodoc


class WireBinanceMarketEvent_Candle extends WireBinanceMarketEvent {
  const WireBinanceMarketEvent_Candle(this.field0): super._();


 final  WireBinanceCandleEvent field0;

/// Create a copy of WireBinanceMarketEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireBinanceMarketEvent_CandleCopyWith<WireBinanceMarketEvent_Candle> get copyWith => _$WireBinanceMarketEvent_CandleCopyWithImpl<WireBinanceMarketEvent_Candle>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBinanceMarketEvent_Candle&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireBinanceMarketEvent.candle(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireBinanceMarketEvent_CandleCopyWith<$Res> implements $WireBinanceMarketEventCopyWith<$Res> {
  factory $WireBinanceMarketEvent_CandleCopyWith(WireBinanceMarketEvent_Candle value, $Res Function(WireBinanceMarketEvent_Candle) _then) = _$WireBinanceMarketEvent_CandleCopyWithImpl;
@useResult
$Res call({
 WireBinanceCandleEvent field0
});




}
/// @nodoc
class _$WireBinanceMarketEvent_CandleCopyWithImpl<$Res>
    implements $WireBinanceMarketEvent_CandleCopyWith<$Res> {
  _$WireBinanceMarketEvent_CandleCopyWithImpl(this._self, this._then);

  final WireBinanceMarketEvent_Candle _self;
  final $Res Function(WireBinanceMarketEvent_Candle) _then;

/// Create a copy of WireBinanceMarketEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireBinanceMarketEvent_Candle(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireBinanceCandleEvent,
  ));
}


}

/// @nodoc


class WireBinanceMarketEvent_Reconnected extends WireBinanceMarketEvent {
  const WireBinanceMarketEvent_Reconnected(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBinanceMarketEvent_Reconnected);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireBinanceMarketEvent.reconnected()';
}


}




/// @nodoc
mixin _$WireBithumbAccountEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBithumbAccountEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireBithumbAccountEvent()';
}


}

/// @nodoc
class $WireBithumbAccountEventCopyWith<$Res>  {
$WireBithumbAccountEventCopyWith(WireBithumbAccountEvent _, $Res Function(WireBithumbAccountEvent) __);
}


/// Adds pattern-matching-related methods to [WireBithumbAccountEvent].
extension WireBithumbAccountEventPatterns on WireBithumbAccountEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireBithumbAccountEvent_Asset value)?  asset,TResult Function( WireBithumbAccountEvent_Order value)?  order,TResult Function( WireBithumbAccountEvent_Reconnected value)?  reconnected,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireBithumbAccountEvent_Asset() when asset != null:
return asset(_that);case WireBithumbAccountEvent_Order() when order != null:
return order(_that);case WireBithumbAccountEvent_Reconnected() when reconnected != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireBithumbAccountEvent_Asset value)  asset,required TResult Function( WireBithumbAccountEvent_Order value)  order,required TResult Function( WireBithumbAccountEvent_Reconnected value)  reconnected,}){
final _that = this;
switch (_that) {
case WireBithumbAccountEvent_Asset():
return asset(_that);case WireBithumbAccountEvent_Order():
return order(_that);case WireBithumbAccountEvent_Reconnected():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireBithumbAccountEvent_Asset value)?  asset,TResult? Function( WireBithumbAccountEvent_Order value)?  order,TResult? Function( WireBithumbAccountEvent_Reconnected value)?  reconnected,}){
final _that = this;
switch (_that) {
case WireBithumbAccountEvent_Asset() when asset != null:
return asset(_that);case WireBithumbAccountEvent_Order() when order != null:
return order(_that);case WireBithumbAccountEvent_Reconnected() when reconnected != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( WireBithumbAssetEvent field0)?  asset,TResult Function( WireBithumbOrderEvent field0)?  order,TResult Function()?  reconnected,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireBithumbAccountEvent_Asset() when asset != null:
return asset(_that.field0);case WireBithumbAccountEvent_Order() when order != null:
return order(_that.field0);case WireBithumbAccountEvent_Reconnected() when reconnected != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( WireBithumbAssetEvent field0)  asset,required TResult Function( WireBithumbOrderEvent field0)  order,required TResult Function()  reconnected,}) {final _that = this;
switch (_that) {
case WireBithumbAccountEvent_Asset():
return asset(_that.field0);case WireBithumbAccountEvent_Order():
return order(_that.field0);case WireBithumbAccountEvent_Reconnected():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( WireBithumbAssetEvent field0)?  asset,TResult? Function( WireBithumbOrderEvent field0)?  order,TResult? Function()?  reconnected,}) {final _that = this;
switch (_that) {
case WireBithumbAccountEvent_Asset() when asset != null:
return asset(_that.field0);case WireBithumbAccountEvent_Order() when order != null:
return order(_that.field0);case WireBithumbAccountEvent_Reconnected() when reconnected != null:
return reconnected();case _:
  return null;

}
}

}

/// @nodoc


class WireBithumbAccountEvent_Asset extends WireBithumbAccountEvent {
  const WireBithumbAccountEvent_Asset(this.field0): super._();


 final  WireBithumbAssetEvent field0;

/// Create a copy of WireBithumbAccountEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireBithumbAccountEvent_AssetCopyWith<WireBithumbAccountEvent_Asset> get copyWith => _$WireBithumbAccountEvent_AssetCopyWithImpl<WireBithumbAccountEvent_Asset>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBithumbAccountEvent_Asset&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireBithumbAccountEvent.asset(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireBithumbAccountEvent_AssetCopyWith<$Res> implements $WireBithumbAccountEventCopyWith<$Res> {
  factory $WireBithumbAccountEvent_AssetCopyWith(WireBithumbAccountEvent_Asset value, $Res Function(WireBithumbAccountEvent_Asset) _then) = _$WireBithumbAccountEvent_AssetCopyWithImpl;
@useResult
$Res call({
 WireBithumbAssetEvent field0
});




}
/// @nodoc
class _$WireBithumbAccountEvent_AssetCopyWithImpl<$Res>
    implements $WireBithumbAccountEvent_AssetCopyWith<$Res> {
  _$WireBithumbAccountEvent_AssetCopyWithImpl(this._self, this._then);

  final WireBithumbAccountEvent_Asset _self;
  final $Res Function(WireBithumbAccountEvent_Asset) _then;

/// Create a copy of WireBithumbAccountEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireBithumbAccountEvent_Asset(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireBithumbAssetEvent,
  ));
}


}

/// @nodoc


class WireBithumbAccountEvent_Order extends WireBithumbAccountEvent {
  const WireBithumbAccountEvent_Order(this.field0): super._();


 final  WireBithumbOrderEvent field0;

/// Create a copy of WireBithumbAccountEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireBithumbAccountEvent_OrderCopyWith<WireBithumbAccountEvent_Order> get copyWith => _$WireBithumbAccountEvent_OrderCopyWithImpl<WireBithumbAccountEvent_Order>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBithumbAccountEvent_Order&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireBithumbAccountEvent.order(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireBithumbAccountEvent_OrderCopyWith<$Res> implements $WireBithumbAccountEventCopyWith<$Res> {
  factory $WireBithumbAccountEvent_OrderCopyWith(WireBithumbAccountEvent_Order value, $Res Function(WireBithumbAccountEvent_Order) _then) = _$WireBithumbAccountEvent_OrderCopyWithImpl;
@useResult
$Res call({
 WireBithumbOrderEvent field0
});




}
/// @nodoc
class _$WireBithumbAccountEvent_OrderCopyWithImpl<$Res>
    implements $WireBithumbAccountEvent_OrderCopyWith<$Res> {
  _$WireBithumbAccountEvent_OrderCopyWithImpl(this._self, this._then);

  final WireBithumbAccountEvent_Order _self;
  final $Res Function(WireBithumbAccountEvent_Order) _then;

/// Create a copy of WireBithumbAccountEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireBithumbAccountEvent_Order(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireBithumbOrderEvent,
  ));
}


}

/// @nodoc


class WireBithumbAccountEvent_Reconnected extends WireBithumbAccountEvent {
  const WireBithumbAccountEvent_Reconnected(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBithumbAccountEvent_Reconnected);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireBithumbAccountEvent.reconnected()';
}


}




/// @nodoc
mixin _$WireBithumbBatchOrderOutcome {

 Object get field0;



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBithumbBatchOrderOutcome&&const DeepCollectionEquality().equals(other.field0, field0));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(field0));

@override
String toString() {
  return 'WireBithumbBatchOrderOutcome(field0: $field0)';
}


}

/// @nodoc
class $WireBithumbBatchOrderOutcomeCopyWith<$Res>  {
$WireBithumbBatchOrderOutcomeCopyWith(WireBithumbBatchOrderOutcome _, $Res Function(WireBithumbBatchOrderOutcome) __);
}


/// Adds pattern-matching-related methods to [WireBithumbBatchOrderOutcome].
extension WireBithumbBatchOrderOutcomePatterns on WireBithumbBatchOrderOutcome {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireBithumbBatchOrderOutcome_Accepted value)?  accepted,TResult Function( WireBithumbBatchOrderOutcome_Rejected value)?  rejected,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireBithumbBatchOrderOutcome_Accepted() when accepted != null:
return accepted(_that);case WireBithumbBatchOrderOutcome_Rejected() when rejected != null:
return rejected(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireBithumbBatchOrderOutcome_Accepted value)  accepted,required TResult Function( WireBithumbBatchOrderOutcome_Rejected value)  rejected,}){
final _that = this;
switch (_that) {
case WireBithumbBatchOrderOutcome_Accepted():
return accepted(_that);case WireBithumbBatchOrderOutcome_Rejected():
return rejected(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireBithumbBatchOrderOutcome_Accepted value)?  accepted,TResult? Function( WireBithumbBatchOrderOutcome_Rejected value)?  rejected,}){
final _that = this;
switch (_that) {
case WireBithumbBatchOrderOutcome_Accepted() when accepted != null:
return accepted(_that);case WireBithumbBatchOrderOutcome_Rejected() when rejected != null:
return rejected(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( WireBithumbBatchOrder field0)?  accepted,TResult Function( WireBithumbBatchOrderFailure field0)?  rejected,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireBithumbBatchOrderOutcome_Accepted() when accepted != null:
return accepted(_that.field0);case WireBithumbBatchOrderOutcome_Rejected() when rejected != null:
return rejected(_that.field0);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( WireBithumbBatchOrder field0)  accepted,required TResult Function( WireBithumbBatchOrderFailure field0)  rejected,}) {final _that = this;
switch (_that) {
case WireBithumbBatchOrderOutcome_Accepted():
return accepted(_that.field0);case WireBithumbBatchOrderOutcome_Rejected():
return rejected(_that.field0);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( WireBithumbBatchOrder field0)?  accepted,TResult? Function( WireBithumbBatchOrderFailure field0)?  rejected,}) {final _that = this;
switch (_that) {
case WireBithumbBatchOrderOutcome_Accepted() when accepted != null:
return accepted(_that.field0);case WireBithumbBatchOrderOutcome_Rejected() when rejected != null:
return rejected(_that.field0);case _:
  return null;

}
}

}

/// @nodoc


class WireBithumbBatchOrderOutcome_Accepted extends WireBithumbBatchOrderOutcome {
  const WireBithumbBatchOrderOutcome_Accepted(this.field0): super._();


@override final  WireBithumbBatchOrder field0;

/// Create a copy of WireBithumbBatchOrderOutcome
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireBithumbBatchOrderOutcome_AcceptedCopyWith<WireBithumbBatchOrderOutcome_Accepted> get copyWith => _$WireBithumbBatchOrderOutcome_AcceptedCopyWithImpl<WireBithumbBatchOrderOutcome_Accepted>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBithumbBatchOrderOutcome_Accepted&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireBithumbBatchOrderOutcome.accepted(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireBithumbBatchOrderOutcome_AcceptedCopyWith<$Res> implements $WireBithumbBatchOrderOutcomeCopyWith<$Res> {
  factory $WireBithumbBatchOrderOutcome_AcceptedCopyWith(WireBithumbBatchOrderOutcome_Accepted value, $Res Function(WireBithumbBatchOrderOutcome_Accepted) _then) = _$WireBithumbBatchOrderOutcome_AcceptedCopyWithImpl;
@useResult
$Res call({
 WireBithumbBatchOrder field0
});




}
/// @nodoc
class _$WireBithumbBatchOrderOutcome_AcceptedCopyWithImpl<$Res>
    implements $WireBithumbBatchOrderOutcome_AcceptedCopyWith<$Res> {
  _$WireBithumbBatchOrderOutcome_AcceptedCopyWithImpl(this._self, this._then);

  final WireBithumbBatchOrderOutcome_Accepted _self;
  final $Res Function(WireBithumbBatchOrderOutcome_Accepted) _then;

/// Create a copy of WireBithumbBatchOrderOutcome
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireBithumbBatchOrderOutcome_Accepted(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireBithumbBatchOrder,
  ));
}


}

/// @nodoc


class WireBithumbBatchOrderOutcome_Rejected extends WireBithumbBatchOrderOutcome {
  const WireBithumbBatchOrderOutcome_Rejected(this.field0): super._();


@override final  WireBithumbBatchOrderFailure field0;

/// Create a copy of WireBithumbBatchOrderOutcome
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireBithumbBatchOrderOutcome_RejectedCopyWith<WireBithumbBatchOrderOutcome_Rejected> get copyWith => _$WireBithumbBatchOrderOutcome_RejectedCopyWithImpl<WireBithumbBatchOrderOutcome_Rejected>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBithumbBatchOrderOutcome_Rejected&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireBithumbBatchOrderOutcome.rejected(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireBithumbBatchOrderOutcome_RejectedCopyWith<$Res> implements $WireBithumbBatchOrderOutcomeCopyWith<$Res> {
  factory $WireBithumbBatchOrderOutcome_RejectedCopyWith(WireBithumbBatchOrderOutcome_Rejected value, $Res Function(WireBithumbBatchOrderOutcome_Rejected) _then) = _$WireBithumbBatchOrderOutcome_RejectedCopyWithImpl;
@useResult
$Res call({
 WireBithumbBatchOrderFailure field0
});




}
/// @nodoc
class _$WireBithumbBatchOrderOutcome_RejectedCopyWithImpl<$Res>
    implements $WireBithumbBatchOrderOutcome_RejectedCopyWith<$Res> {
  _$WireBithumbBatchOrderOutcome_RejectedCopyWithImpl(this._self, this._then);

  final WireBithumbBatchOrderOutcome_Rejected _self;
  final $Res Function(WireBithumbBatchOrderOutcome_Rejected) _then;

/// Create a copy of WireBithumbBatchOrderOutcome
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireBithumbBatchOrderOutcome_Rejected(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireBithumbBatchOrderFailure,
  ));
}


}

/// @nodoc
mixin _$WireBithumbMarketEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBithumbMarketEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireBithumbMarketEvent()';
}


}

/// @nodoc
class $WireBithumbMarketEventCopyWith<$Res>  {
$WireBithumbMarketEventCopyWith(WireBithumbMarketEvent _, $Res Function(WireBithumbMarketEvent) __);
}


/// Adds pattern-matching-related methods to [WireBithumbMarketEvent].
extension WireBithumbMarketEventPatterns on WireBithumbMarketEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireBithumbMarketEvent_Trade value)?  trade,TResult Function( WireBithumbMarketEvent_OrderBook value)?  orderBook,TResult Function( WireBithumbMarketEvent_Ticker value)?  ticker,TResult Function( WireBithumbMarketEvent_Reconnected value)?  reconnected,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireBithumbMarketEvent_Trade() when trade != null:
return trade(_that);case WireBithumbMarketEvent_OrderBook() when orderBook != null:
return orderBook(_that);case WireBithumbMarketEvent_Ticker() when ticker != null:
return ticker(_that);case WireBithumbMarketEvent_Reconnected() when reconnected != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireBithumbMarketEvent_Trade value)  trade,required TResult Function( WireBithumbMarketEvent_OrderBook value)  orderBook,required TResult Function( WireBithumbMarketEvent_Ticker value)  ticker,required TResult Function( WireBithumbMarketEvent_Reconnected value)  reconnected,}){
final _that = this;
switch (_that) {
case WireBithumbMarketEvent_Trade():
return trade(_that);case WireBithumbMarketEvent_OrderBook():
return orderBook(_that);case WireBithumbMarketEvent_Ticker():
return ticker(_that);case WireBithumbMarketEvent_Reconnected():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireBithumbMarketEvent_Trade value)?  trade,TResult? Function( WireBithumbMarketEvent_OrderBook value)?  orderBook,TResult? Function( WireBithumbMarketEvent_Ticker value)?  ticker,TResult? Function( WireBithumbMarketEvent_Reconnected value)?  reconnected,}){
final _that = this;
switch (_that) {
case WireBithumbMarketEvent_Trade() when trade != null:
return trade(_that);case WireBithumbMarketEvent_OrderBook() when orderBook != null:
return orderBook(_that);case WireBithumbMarketEvent_Ticker() when ticker != null:
return ticker(_that);case WireBithumbMarketEvent_Reconnected() when reconnected != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( WireBithumbTradeEvent field0)?  trade,TResult Function( WireBithumbOrderBookEvent field0)?  orderBook,TResult Function( WireBithumbTickerEvent field0)?  ticker,TResult Function()?  reconnected,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireBithumbMarketEvent_Trade() when trade != null:
return trade(_that.field0);case WireBithumbMarketEvent_OrderBook() when orderBook != null:
return orderBook(_that.field0);case WireBithumbMarketEvent_Ticker() when ticker != null:
return ticker(_that.field0);case WireBithumbMarketEvent_Reconnected() when reconnected != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( WireBithumbTradeEvent field0)  trade,required TResult Function( WireBithumbOrderBookEvent field0)  orderBook,required TResult Function( WireBithumbTickerEvent field0)  ticker,required TResult Function()  reconnected,}) {final _that = this;
switch (_that) {
case WireBithumbMarketEvent_Trade():
return trade(_that.field0);case WireBithumbMarketEvent_OrderBook():
return orderBook(_that.field0);case WireBithumbMarketEvent_Ticker():
return ticker(_that.field0);case WireBithumbMarketEvent_Reconnected():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( WireBithumbTradeEvent field0)?  trade,TResult? Function( WireBithumbOrderBookEvent field0)?  orderBook,TResult? Function( WireBithumbTickerEvent field0)?  ticker,TResult? Function()?  reconnected,}) {final _that = this;
switch (_that) {
case WireBithumbMarketEvent_Trade() when trade != null:
return trade(_that.field0);case WireBithumbMarketEvent_OrderBook() when orderBook != null:
return orderBook(_that.field0);case WireBithumbMarketEvent_Ticker() when ticker != null:
return ticker(_that.field0);case WireBithumbMarketEvent_Reconnected() when reconnected != null:
return reconnected();case _:
  return null;

}
}

}

/// @nodoc


class WireBithumbMarketEvent_Trade extends WireBithumbMarketEvent {
  const WireBithumbMarketEvent_Trade(this.field0): super._();


 final  WireBithumbTradeEvent field0;

/// Create a copy of WireBithumbMarketEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireBithumbMarketEvent_TradeCopyWith<WireBithumbMarketEvent_Trade> get copyWith => _$WireBithumbMarketEvent_TradeCopyWithImpl<WireBithumbMarketEvent_Trade>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBithumbMarketEvent_Trade&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireBithumbMarketEvent.trade(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireBithumbMarketEvent_TradeCopyWith<$Res> implements $WireBithumbMarketEventCopyWith<$Res> {
  factory $WireBithumbMarketEvent_TradeCopyWith(WireBithumbMarketEvent_Trade value, $Res Function(WireBithumbMarketEvent_Trade) _then) = _$WireBithumbMarketEvent_TradeCopyWithImpl;
@useResult
$Res call({
 WireBithumbTradeEvent field0
});




}
/// @nodoc
class _$WireBithumbMarketEvent_TradeCopyWithImpl<$Res>
    implements $WireBithumbMarketEvent_TradeCopyWith<$Res> {
  _$WireBithumbMarketEvent_TradeCopyWithImpl(this._self, this._then);

  final WireBithumbMarketEvent_Trade _self;
  final $Res Function(WireBithumbMarketEvent_Trade) _then;

/// Create a copy of WireBithumbMarketEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireBithumbMarketEvent_Trade(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireBithumbTradeEvent,
  ));
}


}

/// @nodoc


class WireBithumbMarketEvent_OrderBook extends WireBithumbMarketEvent {
  const WireBithumbMarketEvent_OrderBook(this.field0): super._();


 final  WireBithumbOrderBookEvent field0;

/// Create a copy of WireBithumbMarketEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireBithumbMarketEvent_OrderBookCopyWith<WireBithumbMarketEvent_OrderBook> get copyWith => _$WireBithumbMarketEvent_OrderBookCopyWithImpl<WireBithumbMarketEvent_OrderBook>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBithumbMarketEvent_OrderBook&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireBithumbMarketEvent.orderBook(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireBithumbMarketEvent_OrderBookCopyWith<$Res> implements $WireBithumbMarketEventCopyWith<$Res> {
  factory $WireBithumbMarketEvent_OrderBookCopyWith(WireBithumbMarketEvent_OrderBook value, $Res Function(WireBithumbMarketEvent_OrderBook) _then) = _$WireBithumbMarketEvent_OrderBookCopyWithImpl;
@useResult
$Res call({
 WireBithumbOrderBookEvent field0
});




}
/// @nodoc
class _$WireBithumbMarketEvent_OrderBookCopyWithImpl<$Res>
    implements $WireBithumbMarketEvent_OrderBookCopyWith<$Res> {
  _$WireBithumbMarketEvent_OrderBookCopyWithImpl(this._self, this._then);

  final WireBithumbMarketEvent_OrderBook _self;
  final $Res Function(WireBithumbMarketEvent_OrderBook) _then;

/// Create a copy of WireBithumbMarketEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireBithumbMarketEvent_OrderBook(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireBithumbOrderBookEvent,
  ));
}


}

/// @nodoc


class WireBithumbMarketEvent_Ticker extends WireBithumbMarketEvent {
  const WireBithumbMarketEvent_Ticker(this.field0): super._();


 final  WireBithumbTickerEvent field0;

/// Create a copy of WireBithumbMarketEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireBithumbMarketEvent_TickerCopyWith<WireBithumbMarketEvent_Ticker> get copyWith => _$WireBithumbMarketEvent_TickerCopyWithImpl<WireBithumbMarketEvent_Ticker>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBithumbMarketEvent_Ticker&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireBithumbMarketEvent.ticker(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireBithumbMarketEvent_TickerCopyWith<$Res> implements $WireBithumbMarketEventCopyWith<$Res> {
  factory $WireBithumbMarketEvent_TickerCopyWith(WireBithumbMarketEvent_Ticker value, $Res Function(WireBithumbMarketEvent_Ticker) _then) = _$WireBithumbMarketEvent_TickerCopyWithImpl;
@useResult
$Res call({
 WireBithumbTickerEvent field0
});




}
/// @nodoc
class _$WireBithumbMarketEvent_TickerCopyWithImpl<$Res>
    implements $WireBithumbMarketEvent_TickerCopyWith<$Res> {
  _$WireBithumbMarketEvent_TickerCopyWithImpl(this._self, this._then);

  final WireBithumbMarketEvent_Ticker _self;
  final $Res Function(WireBithumbMarketEvent_Ticker) _then;

/// Create a copy of WireBithumbMarketEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireBithumbMarketEvent_Ticker(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireBithumbTickerEvent,
  ));
}


}

/// @nodoc


class WireBithumbMarketEvent_Reconnected extends WireBithumbMarketEvent {
  const WireBithumbMarketEvent_Reconnected(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireBithumbMarketEvent_Reconnected);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireBithumbMarketEvent.reconnected()';
}


}




/// @nodoc
mixin _$WireHyperliquidAccountEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidAccountEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireHyperliquidAccountEvent()';
}


}

/// @nodoc
class $WireHyperliquidAccountEventCopyWith<$Res>  {
$WireHyperliquidAccountEventCopyWith(WireHyperliquidAccountEvent _, $Res Function(WireHyperliquidAccountEvent) __);
}


/// Adds pattern-matching-related methods to [WireHyperliquidAccountEvent].
extension WireHyperliquidAccountEventPatterns on WireHyperliquidAccountEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireHyperliquidAccountEvent_OrderUpdate value)?  orderUpdate,TResult Function( WireHyperliquidAccountEvent_SpotState value)?  spotState,TResult Function( WireHyperliquidAccountEvent_Reconnected value)?  reconnected,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireHyperliquidAccountEvent_OrderUpdate() when orderUpdate != null:
return orderUpdate(_that);case WireHyperliquidAccountEvent_SpotState() when spotState != null:
return spotState(_that);case WireHyperliquidAccountEvent_Reconnected() when reconnected != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireHyperliquidAccountEvent_OrderUpdate value)  orderUpdate,required TResult Function( WireHyperliquidAccountEvent_SpotState value)  spotState,required TResult Function( WireHyperliquidAccountEvent_Reconnected value)  reconnected,}){
final _that = this;
switch (_that) {
case WireHyperliquidAccountEvent_OrderUpdate():
return orderUpdate(_that);case WireHyperliquidAccountEvent_SpotState():
return spotState(_that);case WireHyperliquidAccountEvent_Reconnected():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireHyperliquidAccountEvent_OrderUpdate value)?  orderUpdate,TResult? Function( WireHyperliquidAccountEvent_SpotState value)?  spotState,TResult? Function( WireHyperliquidAccountEvent_Reconnected value)?  reconnected,}){
final _that = this;
switch (_that) {
case WireHyperliquidAccountEvent_OrderUpdate() when orderUpdate != null:
return orderUpdate(_that);case WireHyperliquidAccountEvent_SpotState() when spotState != null:
return spotState(_that);case WireHyperliquidAccountEvent_Reconnected() when reconnected != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( WireHyperliquidOrderUpdate field0)?  orderUpdate,TResult Function( WireHyperliquidSpotStateEvent field0)?  spotState,TResult Function()?  reconnected,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireHyperliquidAccountEvent_OrderUpdate() when orderUpdate != null:
return orderUpdate(_that.field0);case WireHyperliquidAccountEvent_SpotState() when spotState != null:
return spotState(_that.field0);case WireHyperliquidAccountEvent_Reconnected() when reconnected != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( WireHyperliquidOrderUpdate field0)  orderUpdate,required TResult Function( WireHyperliquidSpotStateEvent field0)  spotState,required TResult Function()  reconnected,}) {final _that = this;
switch (_that) {
case WireHyperliquidAccountEvent_OrderUpdate():
return orderUpdate(_that.field0);case WireHyperliquidAccountEvent_SpotState():
return spotState(_that.field0);case WireHyperliquidAccountEvent_Reconnected():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( WireHyperliquidOrderUpdate field0)?  orderUpdate,TResult? Function( WireHyperliquidSpotStateEvent field0)?  spotState,TResult? Function()?  reconnected,}) {final _that = this;
switch (_that) {
case WireHyperliquidAccountEvent_OrderUpdate() when orderUpdate != null:
return orderUpdate(_that.field0);case WireHyperliquidAccountEvent_SpotState() when spotState != null:
return spotState(_that.field0);case WireHyperliquidAccountEvent_Reconnected() when reconnected != null:
return reconnected();case _:
  return null;

}
}

}

/// @nodoc


class WireHyperliquidAccountEvent_OrderUpdate extends WireHyperliquidAccountEvent {
  const WireHyperliquidAccountEvent_OrderUpdate(this.field0): super._();


 final  WireHyperliquidOrderUpdate field0;

/// Create a copy of WireHyperliquidAccountEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireHyperliquidAccountEvent_OrderUpdateCopyWith<WireHyperliquidAccountEvent_OrderUpdate> get copyWith => _$WireHyperliquidAccountEvent_OrderUpdateCopyWithImpl<WireHyperliquidAccountEvent_OrderUpdate>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidAccountEvent_OrderUpdate&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireHyperliquidAccountEvent.orderUpdate(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireHyperliquidAccountEvent_OrderUpdateCopyWith<$Res> implements $WireHyperliquidAccountEventCopyWith<$Res> {
  factory $WireHyperliquidAccountEvent_OrderUpdateCopyWith(WireHyperliquidAccountEvent_OrderUpdate value, $Res Function(WireHyperliquidAccountEvent_OrderUpdate) _then) = _$WireHyperliquidAccountEvent_OrderUpdateCopyWithImpl;
@useResult
$Res call({
 WireHyperliquidOrderUpdate field0
});




}
/// @nodoc
class _$WireHyperliquidAccountEvent_OrderUpdateCopyWithImpl<$Res>
    implements $WireHyperliquidAccountEvent_OrderUpdateCopyWith<$Res> {
  _$WireHyperliquidAccountEvent_OrderUpdateCopyWithImpl(this._self, this._then);

  final WireHyperliquidAccountEvent_OrderUpdate _self;
  final $Res Function(WireHyperliquidAccountEvent_OrderUpdate) _then;

/// Create a copy of WireHyperliquidAccountEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireHyperliquidAccountEvent_OrderUpdate(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireHyperliquidOrderUpdate,
  ));
}


}

/// @nodoc


class WireHyperliquidAccountEvent_SpotState extends WireHyperliquidAccountEvent {
  const WireHyperliquidAccountEvent_SpotState(this.field0): super._();


 final  WireHyperliquidSpotStateEvent field0;

/// Create a copy of WireHyperliquidAccountEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireHyperliquidAccountEvent_SpotStateCopyWith<WireHyperliquidAccountEvent_SpotState> get copyWith => _$WireHyperliquidAccountEvent_SpotStateCopyWithImpl<WireHyperliquidAccountEvent_SpotState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidAccountEvent_SpotState&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireHyperliquidAccountEvent.spotState(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireHyperliquidAccountEvent_SpotStateCopyWith<$Res> implements $WireHyperliquidAccountEventCopyWith<$Res> {
  factory $WireHyperliquidAccountEvent_SpotStateCopyWith(WireHyperliquidAccountEvent_SpotState value, $Res Function(WireHyperliquidAccountEvent_SpotState) _then) = _$WireHyperliquidAccountEvent_SpotStateCopyWithImpl;
@useResult
$Res call({
 WireHyperliquidSpotStateEvent field0
});




}
/// @nodoc
class _$WireHyperliquidAccountEvent_SpotStateCopyWithImpl<$Res>
    implements $WireHyperliquidAccountEvent_SpotStateCopyWith<$Res> {
  _$WireHyperliquidAccountEvent_SpotStateCopyWithImpl(this._self, this._then);

  final WireHyperliquidAccountEvent_SpotState _self;
  final $Res Function(WireHyperliquidAccountEvent_SpotState) _then;

/// Create a copy of WireHyperliquidAccountEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireHyperliquidAccountEvent_SpotState(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireHyperliquidSpotStateEvent,
  ));
}


}

/// @nodoc


class WireHyperliquidAccountEvent_Reconnected extends WireHyperliquidAccountEvent {
  const WireHyperliquidAccountEvent_Reconnected(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidAccountEvent_Reconnected);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireHyperliquidAccountEvent.reconnected()';
}


}




/// @nodoc
mixin _$WireHyperliquidMarketEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidMarketEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireHyperliquidMarketEvent()';
}


}

/// @nodoc
class $WireHyperliquidMarketEventCopyWith<$Res>  {
$WireHyperliquidMarketEventCopyWith(WireHyperliquidMarketEvent _, $Res Function(WireHyperliquidMarketEvent) __);
}


/// Adds pattern-matching-related methods to [WireHyperliquidMarketEvent].
extension WireHyperliquidMarketEventPatterns on WireHyperliquidMarketEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireHyperliquidMarketEvent_Trade value)?  trade,TResult Function( WireHyperliquidMarketEvent_OrderBook value)?  orderBook,TResult Function( WireHyperliquidMarketEvent_AssetContext value)?  assetContext,TResult Function( WireHyperliquidMarketEvent_Candle value)?  candle,TResult Function( WireHyperliquidMarketEvent_Reconnected value)?  reconnected,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireHyperliquidMarketEvent_Trade() when trade != null:
return trade(_that);case WireHyperliquidMarketEvent_OrderBook() when orderBook != null:
return orderBook(_that);case WireHyperliquidMarketEvent_AssetContext() when assetContext != null:
return assetContext(_that);case WireHyperliquidMarketEvent_Candle() when candle != null:
return candle(_that);case WireHyperliquidMarketEvent_Reconnected() when reconnected != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireHyperliquidMarketEvent_Trade value)  trade,required TResult Function( WireHyperliquidMarketEvent_OrderBook value)  orderBook,required TResult Function( WireHyperliquidMarketEvent_AssetContext value)  assetContext,required TResult Function( WireHyperliquidMarketEvent_Candle value)  candle,required TResult Function( WireHyperliquidMarketEvent_Reconnected value)  reconnected,}){
final _that = this;
switch (_that) {
case WireHyperliquidMarketEvent_Trade():
return trade(_that);case WireHyperliquidMarketEvent_OrderBook():
return orderBook(_that);case WireHyperliquidMarketEvent_AssetContext():
return assetContext(_that);case WireHyperliquidMarketEvent_Candle():
return candle(_that);case WireHyperliquidMarketEvent_Reconnected():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireHyperliquidMarketEvent_Trade value)?  trade,TResult? Function( WireHyperliquidMarketEvent_OrderBook value)?  orderBook,TResult? Function( WireHyperliquidMarketEvent_AssetContext value)?  assetContext,TResult? Function( WireHyperliquidMarketEvent_Candle value)?  candle,TResult? Function( WireHyperliquidMarketEvent_Reconnected value)?  reconnected,}){
final _that = this;
switch (_that) {
case WireHyperliquidMarketEvent_Trade() when trade != null:
return trade(_that);case WireHyperliquidMarketEvent_OrderBook() when orderBook != null:
return orderBook(_that);case WireHyperliquidMarketEvent_AssetContext() when assetContext != null:
return assetContext(_that);case WireHyperliquidMarketEvent_Candle() when candle != null:
return candle(_that);case WireHyperliquidMarketEvent_Reconnected() when reconnected != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( WireHyperliquidTradeEvent field0)?  trade,TResult Function( WireHyperliquidOrderBookEvent field0)?  orderBook,TResult Function( WireHyperliquidAssetContextEvent field0)?  assetContext,TResult Function( WireHyperliquidCandleEvent field0)?  candle,TResult Function()?  reconnected,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireHyperliquidMarketEvent_Trade() when trade != null:
return trade(_that.field0);case WireHyperliquidMarketEvent_OrderBook() when orderBook != null:
return orderBook(_that.field0);case WireHyperliquidMarketEvent_AssetContext() when assetContext != null:
return assetContext(_that.field0);case WireHyperliquidMarketEvent_Candle() when candle != null:
return candle(_that.field0);case WireHyperliquidMarketEvent_Reconnected() when reconnected != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( WireHyperliquidTradeEvent field0)  trade,required TResult Function( WireHyperliquidOrderBookEvent field0)  orderBook,required TResult Function( WireHyperliquidAssetContextEvent field0)  assetContext,required TResult Function( WireHyperliquidCandleEvent field0)  candle,required TResult Function()  reconnected,}) {final _that = this;
switch (_that) {
case WireHyperliquidMarketEvent_Trade():
return trade(_that.field0);case WireHyperliquidMarketEvent_OrderBook():
return orderBook(_that.field0);case WireHyperliquidMarketEvent_AssetContext():
return assetContext(_that.field0);case WireHyperliquidMarketEvent_Candle():
return candle(_that.field0);case WireHyperliquidMarketEvent_Reconnected():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( WireHyperliquidTradeEvent field0)?  trade,TResult? Function( WireHyperliquidOrderBookEvent field0)?  orderBook,TResult? Function( WireHyperliquidAssetContextEvent field0)?  assetContext,TResult? Function( WireHyperliquidCandleEvent field0)?  candle,TResult? Function()?  reconnected,}) {final _that = this;
switch (_that) {
case WireHyperliquidMarketEvent_Trade() when trade != null:
return trade(_that.field0);case WireHyperliquidMarketEvent_OrderBook() when orderBook != null:
return orderBook(_that.field0);case WireHyperliquidMarketEvent_AssetContext() when assetContext != null:
return assetContext(_that.field0);case WireHyperliquidMarketEvent_Candle() when candle != null:
return candle(_that.field0);case WireHyperliquidMarketEvent_Reconnected() when reconnected != null:
return reconnected();case _:
  return null;

}
}

}

/// @nodoc


class WireHyperliquidMarketEvent_Trade extends WireHyperliquidMarketEvent {
  const WireHyperliquidMarketEvent_Trade(this.field0): super._();


 final  WireHyperliquidTradeEvent field0;

/// Create a copy of WireHyperliquidMarketEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireHyperliquidMarketEvent_TradeCopyWith<WireHyperliquidMarketEvent_Trade> get copyWith => _$WireHyperliquidMarketEvent_TradeCopyWithImpl<WireHyperliquidMarketEvent_Trade>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidMarketEvent_Trade&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireHyperliquidMarketEvent.trade(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireHyperliquidMarketEvent_TradeCopyWith<$Res> implements $WireHyperliquidMarketEventCopyWith<$Res> {
  factory $WireHyperliquidMarketEvent_TradeCopyWith(WireHyperliquidMarketEvent_Trade value, $Res Function(WireHyperliquidMarketEvent_Trade) _then) = _$WireHyperliquidMarketEvent_TradeCopyWithImpl;
@useResult
$Res call({
 WireHyperliquidTradeEvent field0
});




}
/// @nodoc
class _$WireHyperliquidMarketEvent_TradeCopyWithImpl<$Res>
    implements $WireHyperliquidMarketEvent_TradeCopyWith<$Res> {
  _$WireHyperliquidMarketEvent_TradeCopyWithImpl(this._self, this._then);

  final WireHyperliquidMarketEvent_Trade _self;
  final $Res Function(WireHyperliquidMarketEvent_Trade) _then;

/// Create a copy of WireHyperliquidMarketEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireHyperliquidMarketEvent_Trade(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireHyperliquidTradeEvent,
  ));
}


}

/// @nodoc


class WireHyperliquidMarketEvent_OrderBook extends WireHyperliquidMarketEvent {
  const WireHyperliquidMarketEvent_OrderBook(this.field0): super._();


 final  WireHyperliquidOrderBookEvent field0;

/// Create a copy of WireHyperliquidMarketEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireHyperliquidMarketEvent_OrderBookCopyWith<WireHyperliquidMarketEvent_OrderBook> get copyWith => _$WireHyperliquidMarketEvent_OrderBookCopyWithImpl<WireHyperliquidMarketEvent_OrderBook>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidMarketEvent_OrderBook&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireHyperliquidMarketEvent.orderBook(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireHyperliquidMarketEvent_OrderBookCopyWith<$Res> implements $WireHyperliquidMarketEventCopyWith<$Res> {
  factory $WireHyperliquidMarketEvent_OrderBookCopyWith(WireHyperliquidMarketEvent_OrderBook value, $Res Function(WireHyperliquidMarketEvent_OrderBook) _then) = _$WireHyperliquidMarketEvent_OrderBookCopyWithImpl;
@useResult
$Res call({
 WireHyperliquidOrderBookEvent field0
});




}
/// @nodoc
class _$WireHyperliquidMarketEvent_OrderBookCopyWithImpl<$Res>
    implements $WireHyperliquidMarketEvent_OrderBookCopyWith<$Res> {
  _$WireHyperliquidMarketEvent_OrderBookCopyWithImpl(this._self, this._then);

  final WireHyperliquidMarketEvent_OrderBook _self;
  final $Res Function(WireHyperliquidMarketEvent_OrderBook) _then;

/// Create a copy of WireHyperliquidMarketEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireHyperliquidMarketEvent_OrderBook(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireHyperliquidOrderBookEvent,
  ));
}


}

/// @nodoc


class WireHyperliquidMarketEvent_AssetContext extends WireHyperliquidMarketEvent {
  const WireHyperliquidMarketEvent_AssetContext(this.field0): super._();


 final  WireHyperliquidAssetContextEvent field0;

/// Create a copy of WireHyperliquidMarketEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireHyperliquidMarketEvent_AssetContextCopyWith<WireHyperliquidMarketEvent_AssetContext> get copyWith => _$WireHyperliquidMarketEvent_AssetContextCopyWithImpl<WireHyperliquidMarketEvent_AssetContext>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidMarketEvent_AssetContext&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireHyperliquidMarketEvent.assetContext(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireHyperliquidMarketEvent_AssetContextCopyWith<$Res> implements $WireHyperliquidMarketEventCopyWith<$Res> {
  factory $WireHyperliquidMarketEvent_AssetContextCopyWith(WireHyperliquidMarketEvent_AssetContext value, $Res Function(WireHyperliquidMarketEvent_AssetContext) _then) = _$WireHyperliquidMarketEvent_AssetContextCopyWithImpl;
@useResult
$Res call({
 WireHyperliquidAssetContextEvent field0
});




}
/// @nodoc
class _$WireHyperliquidMarketEvent_AssetContextCopyWithImpl<$Res>
    implements $WireHyperliquidMarketEvent_AssetContextCopyWith<$Res> {
  _$WireHyperliquidMarketEvent_AssetContextCopyWithImpl(this._self, this._then);

  final WireHyperliquidMarketEvent_AssetContext _self;
  final $Res Function(WireHyperliquidMarketEvent_AssetContext) _then;

/// Create a copy of WireHyperliquidMarketEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireHyperliquidMarketEvent_AssetContext(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireHyperliquidAssetContextEvent,
  ));
}


}

/// @nodoc


class WireHyperliquidMarketEvent_Candle extends WireHyperliquidMarketEvent {
  const WireHyperliquidMarketEvent_Candle(this.field0): super._();


 final  WireHyperliquidCandleEvent field0;

/// Create a copy of WireHyperliquidMarketEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireHyperliquidMarketEvent_CandleCopyWith<WireHyperliquidMarketEvent_Candle> get copyWith => _$WireHyperliquidMarketEvent_CandleCopyWithImpl<WireHyperliquidMarketEvent_Candle>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidMarketEvent_Candle&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireHyperliquidMarketEvent.candle(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireHyperliquidMarketEvent_CandleCopyWith<$Res> implements $WireHyperliquidMarketEventCopyWith<$Res> {
  factory $WireHyperliquidMarketEvent_CandleCopyWith(WireHyperliquidMarketEvent_Candle value, $Res Function(WireHyperliquidMarketEvent_Candle) _then) = _$WireHyperliquidMarketEvent_CandleCopyWithImpl;
@useResult
$Res call({
 WireHyperliquidCandleEvent field0
});




}
/// @nodoc
class _$WireHyperliquidMarketEvent_CandleCopyWithImpl<$Res>
    implements $WireHyperliquidMarketEvent_CandleCopyWith<$Res> {
  _$WireHyperliquidMarketEvent_CandleCopyWithImpl(this._self, this._then);

  final WireHyperliquidMarketEvent_Candle _self;
  final $Res Function(WireHyperliquidMarketEvent_Candle) _then;

/// Create a copy of WireHyperliquidMarketEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireHyperliquidMarketEvent_Candle(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireHyperliquidCandleEvent,
  ));
}


}

/// @nodoc


class WireHyperliquidMarketEvent_Reconnected extends WireHyperliquidMarketEvent {
  const WireHyperliquidMarketEvent_Reconnected(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidMarketEvent_Reconnected);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireHyperliquidMarketEvent.reconnected()';
}


}




/// @nodoc
mixin _$WireHyperliquidOrderReference {

 Object get field0;



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidOrderReference&&const DeepCollectionEquality().equals(other.field0, field0));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(field0));

@override
String toString() {
  return 'WireHyperliquidOrderReference(field0: $field0)';
}


}

/// @nodoc
class $WireHyperliquidOrderReferenceCopyWith<$Res>  {
$WireHyperliquidOrderReferenceCopyWith(WireHyperliquidOrderReference _, $Res Function(WireHyperliquidOrderReference) __);
}


/// Adds pattern-matching-related methods to [WireHyperliquidOrderReference].
extension WireHyperliquidOrderReferencePatterns on WireHyperliquidOrderReference {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireHyperliquidOrderReference_OrderId value)?  orderId,TResult Function( WireHyperliquidOrderReference_ClientOrderId value)?  clientOrderId,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireHyperliquidOrderReference_OrderId() when orderId != null:
return orderId(_that);case WireHyperliquidOrderReference_ClientOrderId() when clientOrderId != null:
return clientOrderId(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireHyperliquidOrderReference_OrderId value)  orderId,required TResult Function( WireHyperliquidOrderReference_ClientOrderId value)  clientOrderId,}){
final _that = this;
switch (_that) {
case WireHyperliquidOrderReference_OrderId():
return orderId(_that);case WireHyperliquidOrderReference_ClientOrderId():
return clientOrderId(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireHyperliquidOrderReference_OrderId value)?  orderId,TResult? Function( WireHyperliquidOrderReference_ClientOrderId value)?  clientOrderId,}){
final _that = this;
switch (_that) {
case WireHyperliquidOrderReference_OrderId() when orderId != null:
return orderId(_that);case WireHyperliquidOrderReference_ClientOrderId() when clientOrderId != null:
return clientOrderId(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( BigInt field0)?  orderId,TResult Function( String field0)?  clientOrderId,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireHyperliquidOrderReference_OrderId() when orderId != null:
return orderId(_that.field0);case WireHyperliquidOrderReference_ClientOrderId() when clientOrderId != null:
return clientOrderId(_that.field0);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( BigInt field0)  orderId,required TResult Function( String field0)  clientOrderId,}) {final _that = this;
switch (_that) {
case WireHyperliquidOrderReference_OrderId():
return orderId(_that.field0);case WireHyperliquidOrderReference_ClientOrderId():
return clientOrderId(_that.field0);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( BigInt field0)?  orderId,TResult? Function( String field0)?  clientOrderId,}) {final _that = this;
switch (_that) {
case WireHyperliquidOrderReference_OrderId() when orderId != null:
return orderId(_that.field0);case WireHyperliquidOrderReference_ClientOrderId() when clientOrderId != null:
return clientOrderId(_that.field0);case _:
  return null;

}
}

}

/// @nodoc


class WireHyperliquidOrderReference_OrderId extends WireHyperliquidOrderReference {
  const WireHyperliquidOrderReference_OrderId(this.field0): super._();


@override final  BigInt field0;

/// Create a copy of WireHyperliquidOrderReference
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireHyperliquidOrderReference_OrderIdCopyWith<WireHyperliquidOrderReference_OrderId> get copyWith => _$WireHyperliquidOrderReference_OrderIdCopyWithImpl<WireHyperliquidOrderReference_OrderId>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidOrderReference_OrderId&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireHyperliquidOrderReference.orderId(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireHyperliquidOrderReference_OrderIdCopyWith<$Res> implements $WireHyperliquidOrderReferenceCopyWith<$Res> {
  factory $WireHyperliquidOrderReference_OrderIdCopyWith(WireHyperliquidOrderReference_OrderId value, $Res Function(WireHyperliquidOrderReference_OrderId) _then) = _$WireHyperliquidOrderReference_OrderIdCopyWithImpl;
@useResult
$Res call({
 BigInt field0
});




}
/// @nodoc
class _$WireHyperliquidOrderReference_OrderIdCopyWithImpl<$Res>
    implements $WireHyperliquidOrderReference_OrderIdCopyWith<$Res> {
  _$WireHyperliquidOrderReference_OrderIdCopyWithImpl(this._self, this._then);

  final WireHyperliquidOrderReference_OrderId _self;
  final $Res Function(WireHyperliquidOrderReference_OrderId) _then;

/// Create a copy of WireHyperliquidOrderReference
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireHyperliquidOrderReference_OrderId(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as BigInt,
  ));
}


}

/// @nodoc


class WireHyperliquidOrderReference_ClientOrderId extends WireHyperliquidOrderReference {
  const WireHyperliquidOrderReference_ClientOrderId(this.field0): super._();


@override final  String field0;

/// Create a copy of WireHyperliquidOrderReference
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireHyperliquidOrderReference_ClientOrderIdCopyWith<WireHyperliquidOrderReference_ClientOrderId> get copyWith => _$WireHyperliquidOrderReference_ClientOrderIdCopyWithImpl<WireHyperliquidOrderReference_ClientOrderId>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidOrderReference_ClientOrderId&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireHyperliquidOrderReference.clientOrderId(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireHyperliquidOrderReference_ClientOrderIdCopyWith<$Res> implements $WireHyperliquidOrderReferenceCopyWith<$Res> {
  factory $WireHyperliquidOrderReference_ClientOrderIdCopyWith(WireHyperliquidOrderReference_ClientOrderId value, $Res Function(WireHyperliquidOrderReference_ClientOrderId) _then) = _$WireHyperliquidOrderReference_ClientOrderIdCopyWithImpl;
@useResult
$Res call({
 String field0
});




}
/// @nodoc
class _$WireHyperliquidOrderReference_ClientOrderIdCopyWithImpl<$Res>
    implements $WireHyperliquidOrderReference_ClientOrderIdCopyWith<$Res> {
  _$WireHyperliquidOrderReference_ClientOrderIdCopyWithImpl(this._self, this._then);

  final WireHyperliquidOrderReference_ClientOrderId _self;
  final $Res Function(WireHyperliquidOrderReference_ClientOrderId) _then;

/// Create a copy of WireHyperliquidOrderReference
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireHyperliquidOrderReference_ClientOrderId(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$WireHyperliquidOrderStatusResponse {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidOrderStatusResponse);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireHyperliquidOrderStatusResponse()';
}


}

/// @nodoc
class $WireHyperliquidOrderStatusResponseCopyWith<$Res>  {
$WireHyperliquidOrderStatusResponseCopyWith(WireHyperliquidOrderStatusResponse _, $Res Function(WireHyperliquidOrderStatusResponse) __);
}


/// Adds pattern-matching-related methods to [WireHyperliquidOrderStatusResponse].
extension WireHyperliquidOrderStatusResponsePatterns on WireHyperliquidOrderStatusResponse {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireHyperliquidOrderStatusResponse_Order value)?  order,TResult Function( WireHyperliquidOrderStatusResponse_UnknownOrder value)?  unknownOrder,TResult Function( WireHyperliquidOrderStatusResponse_Other value)?  other,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireHyperliquidOrderStatusResponse_Order() when order != null:
return order(_that);case WireHyperliquidOrderStatusResponse_UnknownOrder() when unknownOrder != null:
return unknownOrder(_that);case WireHyperliquidOrderStatusResponse_Other() when other != null:
return other(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireHyperliquidOrderStatusResponse_Order value)  order,required TResult Function( WireHyperliquidOrderStatusResponse_UnknownOrder value)  unknownOrder,required TResult Function( WireHyperliquidOrderStatusResponse_Other value)  other,}){
final _that = this;
switch (_that) {
case WireHyperliquidOrderStatusResponse_Order():
return order(_that);case WireHyperliquidOrderStatusResponse_UnknownOrder():
return unknownOrder(_that);case WireHyperliquidOrderStatusResponse_Other():
return other(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireHyperliquidOrderStatusResponse_Order value)?  order,TResult? Function( WireHyperliquidOrderStatusResponse_UnknownOrder value)?  unknownOrder,TResult? Function( WireHyperliquidOrderStatusResponse_Other value)?  other,}){
final _that = this;
switch (_that) {
case WireHyperliquidOrderStatusResponse_Order() when order != null:
return order(_that);case WireHyperliquidOrderStatusResponse_UnknownOrder() when unknownOrder != null:
return unknownOrder(_that);case WireHyperliquidOrderStatusResponse_Other() when other != null:
return other(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( WireHyperliquidOrderInfo field0)?  order,TResult Function()?  unknownOrder,TResult Function( String status,  String rawJson)?  other,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireHyperliquidOrderStatusResponse_Order() when order != null:
return order(_that.field0);case WireHyperliquidOrderStatusResponse_UnknownOrder() when unknownOrder != null:
return unknownOrder();case WireHyperliquidOrderStatusResponse_Other() when other != null:
return other(_that.status,_that.rawJson);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( WireHyperliquidOrderInfo field0)  order,required TResult Function()  unknownOrder,required TResult Function( String status,  String rawJson)  other,}) {final _that = this;
switch (_that) {
case WireHyperliquidOrderStatusResponse_Order():
return order(_that.field0);case WireHyperliquidOrderStatusResponse_UnknownOrder():
return unknownOrder();case WireHyperliquidOrderStatusResponse_Other():
return other(_that.status,_that.rawJson);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( WireHyperliquidOrderInfo field0)?  order,TResult? Function()?  unknownOrder,TResult? Function( String status,  String rawJson)?  other,}) {final _that = this;
switch (_that) {
case WireHyperliquidOrderStatusResponse_Order() when order != null:
return order(_that.field0);case WireHyperliquidOrderStatusResponse_UnknownOrder() when unknownOrder != null:
return unknownOrder();case WireHyperliquidOrderStatusResponse_Other() when other != null:
return other(_that.status,_that.rawJson);case _:
  return null;

}
}

}

/// @nodoc


class WireHyperliquidOrderStatusResponse_Order extends WireHyperliquidOrderStatusResponse {
  const WireHyperliquidOrderStatusResponse_Order(this.field0): super._();


 final  WireHyperliquidOrderInfo field0;

/// Create a copy of WireHyperliquidOrderStatusResponse
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireHyperliquidOrderStatusResponse_OrderCopyWith<WireHyperliquidOrderStatusResponse_Order> get copyWith => _$WireHyperliquidOrderStatusResponse_OrderCopyWithImpl<WireHyperliquidOrderStatusResponse_Order>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidOrderStatusResponse_Order&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireHyperliquidOrderStatusResponse.order(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireHyperliquidOrderStatusResponse_OrderCopyWith<$Res> implements $WireHyperliquidOrderStatusResponseCopyWith<$Res> {
  factory $WireHyperliquidOrderStatusResponse_OrderCopyWith(WireHyperliquidOrderStatusResponse_Order value, $Res Function(WireHyperliquidOrderStatusResponse_Order) _then) = _$WireHyperliquidOrderStatusResponse_OrderCopyWithImpl;
@useResult
$Res call({
 WireHyperliquidOrderInfo field0
});




}
/// @nodoc
class _$WireHyperliquidOrderStatusResponse_OrderCopyWithImpl<$Res>
    implements $WireHyperliquidOrderStatusResponse_OrderCopyWith<$Res> {
  _$WireHyperliquidOrderStatusResponse_OrderCopyWithImpl(this._self, this._then);

  final WireHyperliquidOrderStatusResponse_Order _self;
  final $Res Function(WireHyperliquidOrderStatusResponse_Order) _then;

/// Create a copy of WireHyperliquidOrderStatusResponse
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireHyperliquidOrderStatusResponse_Order(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireHyperliquidOrderInfo,
  ));
}


}

/// @nodoc


class WireHyperliquidOrderStatusResponse_UnknownOrder extends WireHyperliquidOrderStatusResponse {
  const WireHyperliquidOrderStatusResponse_UnknownOrder(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidOrderStatusResponse_UnknownOrder);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireHyperliquidOrderStatusResponse.unknownOrder()';
}


}




/// @nodoc


class WireHyperliquidOrderStatusResponse_Other extends WireHyperliquidOrderStatusResponse {
  const WireHyperliquidOrderStatusResponse_Other({required this.status, required this.rawJson}): super._();


 final  String status;
 final  String rawJson;

/// Create a copy of WireHyperliquidOrderStatusResponse
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireHyperliquidOrderStatusResponse_OtherCopyWith<WireHyperliquidOrderStatusResponse_Other> get copyWith => _$WireHyperliquidOrderStatusResponse_OtherCopyWithImpl<WireHyperliquidOrderStatusResponse_Other>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidOrderStatusResponse_Other&&(identical(other.status, status) || other.status == status)&&(identical(other.rawJson, rawJson) || other.rawJson == rawJson));
}


@override
int get hashCode => Object.hash(runtimeType,status,rawJson);

@override
String toString() {
  return 'WireHyperliquidOrderStatusResponse.other(status: $status, rawJson: $rawJson)';
}


}

/// @nodoc
abstract mixin class $WireHyperliquidOrderStatusResponse_OtherCopyWith<$Res> implements $WireHyperliquidOrderStatusResponseCopyWith<$Res> {
  factory $WireHyperliquidOrderStatusResponse_OtherCopyWith(WireHyperliquidOrderStatusResponse_Other value, $Res Function(WireHyperliquidOrderStatusResponse_Other) _then) = _$WireHyperliquidOrderStatusResponse_OtherCopyWithImpl;
@useResult
$Res call({
 String status, String rawJson
});




}
/// @nodoc
class _$WireHyperliquidOrderStatusResponse_OtherCopyWithImpl<$Res>
    implements $WireHyperliquidOrderStatusResponse_OtherCopyWith<$Res> {
  _$WireHyperliquidOrderStatusResponse_OtherCopyWithImpl(this._self, this._then);

  final WireHyperliquidOrderStatusResponse_Other _self;
  final $Res Function(WireHyperliquidOrderStatusResponse_Other) _then;

/// Create a copy of WireHyperliquidOrderStatusResponse
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? status = null,Object? rawJson = null,}) {
  return _then(WireHyperliquidOrderStatusResponse_Other(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as String,rawJson: null == rawJson ? _self.rawJson : rawJson // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$WireHyperliquidUserRole {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidUserRole);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireHyperliquidUserRole()';
}


}

/// @nodoc
class $WireHyperliquidUserRoleCopyWith<$Res>  {
$WireHyperliquidUserRoleCopyWith(WireHyperliquidUserRole _, $Res Function(WireHyperliquidUserRole) __);
}


/// Adds pattern-matching-related methods to [WireHyperliquidUserRole].
extension WireHyperliquidUserRolePatterns on WireHyperliquidUserRole {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireHyperliquidUserRole_User value)?  user,TResult Function( WireHyperliquidUserRole_Agent value)?  agent,TResult Function( WireHyperliquidUserRole_Vault value)?  vault,TResult Function( WireHyperliquidUserRole_SubAccount value)?  subAccount,TResult Function( WireHyperliquidUserRole_Missing value)?  missing,TResult Function( WireHyperliquidUserRole_Other value)?  other,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireHyperliquidUserRole_User() when user != null:
return user(_that);case WireHyperliquidUserRole_Agent() when agent != null:
return agent(_that);case WireHyperliquidUserRole_Vault() when vault != null:
return vault(_that);case WireHyperliquidUserRole_SubAccount() when subAccount != null:
return subAccount(_that);case WireHyperliquidUserRole_Missing() when missing != null:
return missing(_that);case WireHyperliquidUserRole_Other() when other != null:
return other(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireHyperliquidUserRole_User value)  user,required TResult Function( WireHyperliquidUserRole_Agent value)  agent,required TResult Function( WireHyperliquidUserRole_Vault value)  vault,required TResult Function( WireHyperliquidUserRole_SubAccount value)  subAccount,required TResult Function( WireHyperliquidUserRole_Missing value)  missing,required TResult Function( WireHyperliquidUserRole_Other value)  other,}){
final _that = this;
switch (_that) {
case WireHyperliquidUserRole_User():
return user(_that);case WireHyperliquidUserRole_Agent():
return agent(_that);case WireHyperliquidUserRole_Vault():
return vault(_that);case WireHyperliquidUserRole_SubAccount():
return subAccount(_that);case WireHyperliquidUserRole_Missing():
return missing(_that);case WireHyperliquidUserRole_Other():
return other(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireHyperliquidUserRole_User value)?  user,TResult? Function( WireHyperliquidUserRole_Agent value)?  agent,TResult? Function( WireHyperliquidUserRole_Vault value)?  vault,TResult? Function( WireHyperliquidUserRole_SubAccount value)?  subAccount,TResult? Function( WireHyperliquidUserRole_Missing value)?  missing,TResult? Function( WireHyperliquidUserRole_Other value)?  other,}){
final _that = this;
switch (_that) {
case WireHyperliquidUserRole_User() when user != null:
return user(_that);case WireHyperliquidUserRole_Agent() when agent != null:
return agent(_that);case WireHyperliquidUserRole_Vault() when vault != null:
return vault(_that);case WireHyperliquidUserRole_SubAccount() when subAccount != null:
return subAccount(_that);case WireHyperliquidUserRole_Missing() when missing != null:
return missing(_that);case WireHyperliquidUserRole_Other() when other != null:
return other(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  user,TResult Function( String? user)?  agent,TResult Function()?  vault,TResult Function( String? master)?  subAccount,TResult Function()?  missing,TResult Function( String role,  String? dataJson)?  other,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireHyperliquidUserRole_User() when user != null:
return user();case WireHyperliquidUserRole_Agent() when agent != null:
return agent(_that.user);case WireHyperliquidUserRole_Vault() when vault != null:
return vault();case WireHyperliquidUserRole_SubAccount() when subAccount != null:
return subAccount(_that.master);case WireHyperliquidUserRole_Missing() when missing != null:
return missing();case WireHyperliquidUserRole_Other() when other != null:
return other(_that.role,_that.dataJson);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  user,required TResult Function( String? user)  agent,required TResult Function()  vault,required TResult Function( String? master)  subAccount,required TResult Function()  missing,required TResult Function( String role,  String? dataJson)  other,}) {final _that = this;
switch (_that) {
case WireHyperliquidUserRole_User():
return user();case WireHyperliquidUserRole_Agent():
return agent(_that.user);case WireHyperliquidUserRole_Vault():
return vault();case WireHyperliquidUserRole_SubAccount():
return subAccount(_that.master);case WireHyperliquidUserRole_Missing():
return missing();case WireHyperliquidUserRole_Other():
return other(_that.role,_that.dataJson);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  user,TResult? Function( String? user)?  agent,TResult? Function()?  vault,TResult? Function( String? master)?  subAccount,TResult? Function()?  missing,TResult? Function( String role,  String? dataJson)?  other,}) {final _that = this;
switch (_that) {
case WireHyperliquidUserRole_User() when user != null:
return user();case WireHyperliquidUserRole_Agent() when agent != null:
return agent(_that.user);case WireHyperliquidUserRole_Vault() when vault != null:
return vault();case WireHyperliquidUserRole_SubAccount() when subAccount != null:
return subAccount(_that.master);case WireHyperliquidUserRole_Missing() when missing != null:
return missing();case WireHyperliquidUserRole_Other() when other != null:
return other(_that.role,_that.dataJson);case _:
  return null;

}
}

}

/// @nodoc


class WireHyperliquidUserRole_User extends WireHyperliquidUserRole {
  const WireHyperliquidUserRole_User(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidUserRole_User);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireHyperliquidUserRole.user()';
}


}




/// @nodoc


class WireHyperliquidUserRole_Agent extends WireHyperliquidUserRole {
  const WireHyperliquidUserRole_Agent({this.user}): super._();


 final  String? user;

/// Create a copy of WireHyperliquidUserRole
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireHyperliquidUserRole_AgentCopyWith<WireHyperliquidUserRole_Agent> get copyWith => _$WireHyperliquidUserRole_AgentCopyWithImpl<WireHyperliquidUserRole_Agent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidUserRole_Agent&&(identical(other.user, user) || other.user == user));
}


@override
int get hashCode => Object.hash(runtimeType,user);

@override
String toString() {
  return 'WireHyperliquidUserRole.agent(user: $user)';
}


}

/// @nodoc
abstract mixin class $WireHyperliquidUserRole_AgentCopyWith<$Res> implements $WireHyperliquidUserRoleCopyWith<$Res> {
  factory $WireHyperliquidUserRole_AgentCopyWith(WireHyperliquidUserRole_Agent value, $Res Function(WireHyperliquidUserRole_Agent) _then) = _$WireHyperliquidUserRole_AgentCopyWithImpl;
@useResult
$Res call({
 String? user
});




}
/// @nodoc
class _$WireHyperliquidUserRole_AgentCopyWithImpl<$Res>
    implements $WireHyperliquidUserRole_AgentCopyWith<$Res> {
  _$WireHyperliquidUserRole_AgentCopyWithImpl(this._self, this._then);

  final WireHyperliquidUserRole_Agent _self;
  final $Res Function(WireHyperliquidUserRole_Agent) _then;

/// Create a copy of WireHyperliquidUserRole
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? user = freezed,}) {
  return _then(WireHyperliquidUserRole_Agent(
user: freezed == user ? _self.user : user // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc


class WireHyperliquidUserRole_Vault extends WireHyperliquidUserRole {
  const WireHyperliquidUserRole_Vault(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidUserRole_Vault);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireHyperliquidUserRole.vault()';
}


}




/// @nodoc


class WireHyperliquidUserRole_SubAccount extends WireHyperliquidUserRole {
  const WireHyperliquidUserRole_SubAccount({this.master}): super._();


 final  String? master;

/// Create a copy of WireHyperliquidUserRole
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireHyperliquidUserRole_SubAccountCopyWith<WireHyperliquidUserRole_SubAccount> get copyWith => _$WireHyperliquidUserRole_SubAccountCopyWithImpl<WireHyperliquidUserRole_SubAccount>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidUserRole_SubAccount&&(identical(other.master, master) || other.master == master));
}


@override
int get hashCode => Object.hash(runtimeType,master);

@override
String toString() {
  return 'WireHyperliquidUserRole.subAccount(master: $master)';
}


}

/// @nodoc
abstract mixin class $WireHyperliquidUserRole_SubAccountCopyWith<$Res> implements $WireHyperliquidUserRoleCopyWith<$Res> {
  factory $WireHyperliquidUserRole_SubAccountCopyWith(WireHyperliquidUserRole_SubAccount value, $Res Function(WireHyperliquidUserRole_SubAccount) _then) = _$WireHyperliquidUserRole_SubAccountCopyWithImpl;
@useResult
$Res call({
 String? master
});




}
/// @nodoc
class _$WireHyperliquidUserRole_SubAccountCopyWithImpl<$Res>
    implements $WireHyperliquidUserRole_SubAccountCopyWith<$Res> {
  _$WireHyperliquidUserRole_SubAccountCopyWithImpl(this._self, this._then);

  final WireHyperliquidUserRole_SubAccount _self;
  final $Res Function(WireHyperliquidUserRole_SubAccount) _then;

/// Create a copy of WireHyperliquidUserRole
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? master = freezed,}) {
  return _then(WireHyperliquidUserRole_SubAccount(
master: freezed == master ? _self.master : master // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc


class WireHyperliquidUserRole_Missing extends WireHyperliquidUserRole {
  const WireHyperliquidUserRole_Missing(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidUserRole_Missing);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireHyperliquidUserRole.missing()';
}


}




/// @nodoc


class WireHyperliquidUserRole_Other extends WireHyperliquidUserRole {
  const WireHyperliquidUserRole_Other({required this.role, this.dataJson}): super._();


 final  String role;
 final  String? dataJson;

/// Create a copy of WireHyperliquidUserRole
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireHyperliquidUserRole_OtherCopyWith<WireHyperliquidUserRole_Other> get copyWith => _$WireHyperliquidUserRole_OtherCopyWithImpl<WireHyperliquidUserRole_Other>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireHyperliquidUserRole_Other&&(identical(other.role, role) || other.role == role)&&(identical(other.dataJson, dataJson) || other.dataJson == dataJson));
}


@override
int get hashCode => Object.hash(runtimeType,role,dataJson);

@override
String toString() {
  return 'WireHyperliquidUserRole.other(role: $role, dataJson: $dataJson)';
}


}

/// @nodoc
abstract mixin class $WireHyperliquidUserRole_OtherCopyWith<$Res> implements $WireHyperliquidUserRoleCopyWith<$Res> {
  factory $WireHyperliquidUserRole_OtherCopyWith(WireHyperliquidUserRole_Other value, $Res Function(WireHyperliquidUserRole_Other) _then) = _$WireHyperliquidUserRole_OtherCopyWithImpl;
@useResult
$Res call({
 String role, String? dataJson
});




}
/// @nodoc
class _$WireHyperliquidUserRole_OtherCopyWithImpl<$Res>
    implements $WireHyperliquidUserRole_OtherCopyWith<$Res> {
  _$WireHyperliquidUserRole_OtherCopyWithImpl(this._self, this._then);

  final WireHyperliquidUserRole_Other _self;
  final $Res Function(WireHyperliquidUserRole_Other) _then;

/// Create a copy of WireHyperliquidUserRole
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? role = null,Object? dataJson = freezed,}) {
  return _then(WireHyperliquidUserRole_Other(
role: null == role ? _self.role : role // ignore: cast_nullable_to_non_nullable
as String,dataJson: freezed == dataJson ? _self.dataJson : dataJson // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc
mixin _$WireTransferDestination {

 Object get field0;



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireTransferDestination&&const DeepCollectionEquality().equals(other.field0, field0));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(field0));

@override
String toString() {
  return 'WireTransferDestination(field0: $field0)';
}


}

/// @nodoc
class $WireTransferDestinationCopyWith<$Res>  {
$WireTransferDestinationCopyWith(WireTransferDestination _, $Res Function(WireTransferDestination) __);
}


/// Adds pattern-matching-related methods to [WireTransferDestination].
extension WireTransferDestinationPatterns on WireTransferDestination {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireTransferDestination_Exchange value)?  exchange,TResult Function( WireTransferDestination_Chain value)?  chain,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireTransferDestination_Exchange() when exchange != null:
return exchange(_that);case WireTransferDestination_Chain() when chain != null:
return chain(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireTransferDestination_Exchange value)  exchange,required TResult Function( WireTransferDestination_Chain value)  chain,}){
final _that = this;
switch (_that) {
case WireTransferDestination_Exchange():
return exchange(_that);case WireTransferDestination_Chain():
return chain(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireTransferDestination_Exchange value)?  exchange,TResult? Function( WireTransferDestination_Chain value)?  chain,}){
final _that = this;
switch (_that) {
case WireTransferDestination_Exchange() when exchange != null:
return exchange(_that);case WireTransferDestination_Chain() when chain != null:
return chain(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( WireExchangeDestination field0)?  exchange,TResult Function( WireChainDestination field0)?  chain,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireTransferDestination_Exchange() when exchange != null:
return exchange(_that.field0);case WireTransferDestination_Chain() when chain != null:
return chain(_that.field0);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( WireExchangeDestination field0)  exchange,required TResult Function( WireChainDestination field0)  chain,}) {final _that = this;
switch (_that) {
case WireTransferDestination_Exchange():
return exchange(_that.field0);case WireTransferDestination_Chain():
return chain(_that.field0);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( WireExchangeDestination field0)?  exchange,TResult? Function( WireChainDestination field0)?  chain,}) {final _that = this;
switch (_that) {
case WireTransferDestination_Exchange() when exchange != null:
return exchange(_that.field0);case WireTransferDestination_Chain() when chain != null:
return chain(_that.field0);case _:
  return null;

}
}

}

/// @nodoc


class WireTransferDestination_Exchange extends WireTransferDestination {
  const WireTransferDestination_Exchange(this.field0): super._();


@override final  WireExchangeDestination field0;

/// Create a copy of WireTransferDestination
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireTransferDestination_ExchangeCopyWith<WireTransferDestination_Exchange> get copyWith => _$WireTransferDestination_ExchangeCopyWithImpl<WireTransferDestination_Exchange>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireTransferDestination_Exchange&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireTransferDestination.exchange(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireTransferDestination_ExchangeCopyWith<$Res> implements $WireTransferDestinationCopyWith<$Res> {
  factory $WireTransferDestination_ExchangeCopyWith(WireTransferDestination_Exchange value, $Res Function(WireTransferDestination_Exchange) _then) = _$WireTransferDestination_ExchangeCopyWithImpl;
@useResult
$Res call({
 WireExchangeDestination field0
});




}
/// @nodoc
class _$WireTransferDestination_ExchangeCopyWithImpl<$Res>
    implements $WireTransferDestination_ExchangeCopyWith<$Res> {
  _$WireTransferDestination_ExchangeCopyWithImpl(this._self, this._then);

  final WireTransferDestination_Exchange _self;
  final $Res Function(WireTransferDestination_Exchange) _then;

/// Create a copy of WireTransferDestination
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireTransferDestination_Exchange(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireExchangeDestination,
  ));
}


}

/// @nodoc


class WireTransferDestination_Chain extends WireTransferDestination {
  const WireTransferDestination_Chain(this.field0): super._();


@override final  WireChainDestination field0;

/// Create a copy of WireTransferDestination
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireTransferDestination_ChainCopyWith<WireTransferDestination_Chain> get copyWith => _$WireTransferDestination_ChainCopyWithImpl<WireTransferDestination_Chain>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireTransferDestination_Chain&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireTransferDestination.chain(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireTransferDestination_ChainCopyWith<$Res> implements $WireTransferDestinationCopyWith<$Res> {
  factory $WireTransferDestination_ChainCopyWith(WireTransferDestination_Chain value, $Res Function(WireTransferDestination_Chain) _then) = _$WireTransferDestination_ChainCopyWithImpl;
@useResult
$Res call({
 WireChainDestination field0
});




}
/// @nodoc
class _$WireTransferDestination_ChainCopyWithImpl<$Res>
    implements $WireTransferDestination_ChainCopyWith<$Res> {
  _$WireTransferDestination_ChainCopyWithImpl(this._self, this._then);

  final WireTransferDestination_Chain _self;
  final $Res Function(WireTransferDestination_Chain) _then;

/// Create a copy of WireTransferDestination
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireTransferDestination_Chain(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireChainDestination,
  ));
}


}

/// @nodoc
mixin _$WireTravelRuleRequirement {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireTravelRuleRequirement);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireTravelRuleRequirement()';
}


}

/// @nodoc
class $WireTravelRuleRequirementCopyWith<$Res>  {
$WireTravelRuleRequirementCopyWith(WireTravelRuleRequirement _, $Res Function(WireTravelRuleRequirement) __);
}


/// Adds pattern-matching-related methods to [WireTravelRuleRequirement].
extension WireTravelRuleRequirementPatterns on WireTravelRuleRequirement {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireTravelRuleRequirement_NotRequired value)?  notRequired,TResult Function( WireTravelRuleRequirement_Required value)?  required_,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireTravelRuleRequirement_NotRequired() when notRequired != null:
return notRequired(_that);case WireTravelRuleRequirement_Required() when required_ != null:
return required_(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireTravelRuleRequirement_NotRequired value)  notRequired,required TResult Function( WireTravelRuleRequirement_Required value)  required_,}){
final _that = this;
switch (_that) {
case WireTravelRuleRequirement_NotRequired():
return notRequired(_that);case WireTravelRuleRequirement_Required():
return required_(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireTravelRuleRequirement_NotRequired value)?  notRequired,TResult? Function( WireTravelRuleRequirement_Required value)?  required_,}){
final _that = this;
switch (_that) {
case WireTravelRuleRequirement_NotRequired() when notRequired != null:
return notRequired(_that);case WireTravelRuleRequirement_Required() when required_ != null:
return required_(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  notRequired,TResult Function( String? consentUrl)?  required_,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireTravelRuleRequirement_NotRequired() when notRequired != null:
return notRequired();case WireTravelRuleRequirement_Required() when required_ != null:
return required_(_that.consentUrl);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  notRequired,required TResult Function( String? consentUrl)  required_,}) {final _that = this;
switch (_that) {
case WireTravelRuleRequirement_NotRequired():
return notRequired();case WireTravelRuleRequirement_Required():
return required_(_that.consentUrl);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  notRequired,TResult? Function( String? consentUrl)?  required_,}) {final _that = this;
switch (_that) {
case WireTravelRuleRequirement_NotRequired() when notRequired != null:
return notRequired();case WireTravelRuleRequirement_Required() when required_ != null:
return required_(_that.consentUrl);case _:
  return null;

}
}

}

/// @nodoc


class WireTravelRuleRequirement_NotRequired extends WireTravelRuleRequirement {
  const WireTravelRuleRequirement_NotRequired(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireTravelRuleRequirement_NotRequired);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireTravelRuleRequirement.notRequired()';
}


}




/// @nodoc


class WireTravelRuleRequirement_Required extends WireTravelRuleRequirement {
  const WireTravelRuleRequirement_Required({this.consentUrl}): super._();


 final  String? consentUrl;

/// Create a copy of WireTravelRuleRequirement
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireTravelRuleRequirement_RequiredCopyWith<WireTravelRuleRequirement_Required> get copyWith => _$WireTravelRuleRequirement_RequiredCopyWithImpl<WireTravelRuleRequirement_Required>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireTravelRuleRequirement_Required&&(identical(other.consentUrl, consentUrl) || other.consentUrl == consentUrl));
}


@override
int get hashCode => Object.hash(runtimeType,consentUrl);

@override
String toString() {
  return 'WireTravelRuleRequirement.required_(consentUrl: $consentUrl)';
}


}

/// @nodoc
abstract mixin class $WireTravelRuleRequirement_RequiredCopyWith<$Res> implements $WireTravelRuleRequirementCopyWith<$Res> {
  factory $WireTravelRuleRequirement_RequiredCopyWith(WireTravelRuleRequirement_Required value, $Res Function(WireTravelRuleRequirement_Required) _then) = _$WireTravelRuleRequirement_RequiredCopyWithImpl;
@useResult
$Res call({
 String? consentUrl
});




}
/// @nodoc
class _$WireTravelRuleRequirement_RequiredCopyWithImpl<$Res>
    implements $WireTravelRuleRequirement_RequiredCopyWith<$Res> {
  _$WireTravelRuleRequirement_RequiredCopyWithImpl(this._self, this._then);

  final WireTravelRuleRequirement_Required _self;
  final $Res Function(WireTravelRuleRequirement_Required) _then;

/// Create a copy of WireTravelRuleRequirement
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? consentUrl = freezed,}) {
  return _then(WireTravelRuleRequirement_Required(
consentUrl: freezed == consentUrl ? _self.consentUrl : consentUrl // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc
mixin _$WireUpbitAccountStreamEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitAccountStreamEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireUpbitAccountStreamEvent()';
}


}

/// @nodoc
class $WireUpbitAccountStreamEventCopyWith<$Res>  {
$WireUpbitAccountStreamEventCopyWith(WireUpbitAccountStreamEvent _, $Res Function(WireUpbitAccountStreamEvent) __);
}


/// Adds pattern-matching-related methods to [WireUpbitAccountStreamEvent].
extension WireUpbitAccountStreamEventPatterns on WireUpbitAccountStreamEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireUpbitAccountStreamEvent_Asset value)?  asset,TResult Function( WireUpbitAccountStreamEvent_Order value)?  order,TResult Function( WireUpbitAccountStreamEvent_Reconnected value)?  reconnected,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireUpbitAccountStreamEvent_Asset() when asset != null:
return asset(_that);case WireUpbitAccountStreamEvent_Order() when order != null:
return order(_that);case WireUpbitAccountStreamEvent_Reconnected() when reconnected != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireUpbitAccountStreamEvent_Asset value)  asset,required TResult Function( WireUpbitAccountStreamEvent_Order value)  order,required TResult Function( WireUpbitAccountStreamEvent_Reconnected value)  reconnected,}){
final _that = this;
switch (_that) {
case WireUpbitAccountStreamEvent_Asset():
return asset(_that);case WireUpbitAccountStreamEvent_Order():
return order(_that);case WireUpbitAccountStreamEvent_Reconnected():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireUpbitAccountStreamEvent_Asset value)?  asset,TResult? Function( WireUpbitAccountStreamEvent_Order value)?  order,TResult? Function( WireUpbitAccountStreamEvent_Reconnected value)?  reconnected,}){
final _that = this;
switch (_that) {
case WireUpbitAccountStreamEvent_Asset() when asset != null:
return asset(_that);case WireUpbitAccountStreamEvent_Order() when order != null:
return order(_that);case WireUpbitAccountStreamEvent_Reconnected() when reconnected != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( WireUpbitAssetStreamEvent field0)?  asset,TResult Function( WireUpbitOrderStreamEvent field0)?  order,TResult Function()?  reconnected,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireUpbitAccountStreamEvent_Asset() when asset != null:
return asset(_that.field0);case WireUpbitAccountStreamEvent_Order() when order != null:
return order(_that.field0);case WireUpbitAccountStreamEvent_Reconnected() when reconnected != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( WireUpbitAssetStreamEvent field0)  asset,required TResult Function( WireUpbitOrderStreamEvent field0)  order,required TResult Function()  reconnected,}) {final _that = this;
switch (_that) {
case WireUpbitAccountStreamEvent_Asset():
return asset(_that.field0);case WireUpbitAccountStreamEvent_Order():
return order(_that.field0);case WireUpbitAccountStreamEvent_Reconnected():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( WireUpbitAssetStreamEvent field0)?  asset,TResult? Function( WireUpbitOrderStreamEvent field0)?  order,TResult? Function()?  reconnected,}) {final _that = this;
switch (_that) {
case WireUpbitAccountStreamEvent_Asset() when asset != null:
return asset(_that.field0);case WireUpbitAccountStreamEvent_Order() when order != null:
return order(_that.field0);case WireUpbitAccountStreamEvent_Reconnected() when reconnected != null:
return reconnected();case _:
  return null;

}
}

}

/// @nodoc


class WireUpbitAccountStreamEvent_Asset extends WireUpbitAccountStreamEvent {
  const WireUpbitAccountStreamEvent_Asset(this.field0): super._();


 final  WireUpbitAssetStreamEvent field0;

/// Create a copy of WireUpbitAccountStreamEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireUpbitAccountStreamEvent_AssetCopyWith<WireUpbitAccountStreamEvent_Asset> get copyWith => _$WireUpbitAccountStreamEvent_AssetCopyWithImpl<WireUpbitAccountStreamEvent_Asset>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitAccountStreamEvent_Asset&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireUpbitAccountStreamEvent.asset(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireUpbitAccountStreamEvent_AssetCopyWith<$Res> implements $WireUpbitAccountStreamEventCopyWith<$Res> {
  factory $WireUpbitAccountStreamEvent_AssetCopyWith(WireUpbitAccountStreamEvent_Asset value, $Res Function(WireUpbitAccountStreamEvent_Asset) _then) = _$WireUpbitAccountStreamEvent_AssetCopyWithImpl;
@useResult
$Res call({
 WireUpbitAssetStreamEvent field0
});




}
/// @nodoc
class _$WireUpbitAccountStreamEvent_AssetCopyWithImpl<$Res>
    implements $WireUpbitAccountStreamEvent_AssetCopyWith<$Res> {
  _$WireUpbitAccountStreamEvent_AssetCopyWithImpl(this._self, this._then);

  final WireUpbitAccountStreamEvent_Asset _self;
  final $Res Function(WireUpbitAccountStreamEvent_Asset) _then;

/// Create a copy of WireUpbitAccountStreamEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireUpbitAccountStreamEvent_Asset(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireUpbitAssetStreamEvent,
  ));
}


}

/// @nodoc


class WireUpbitAccountStreamEvent_Order extends WireUpbitAccountStreamEvent {
  const WireUpbitAccountStreamEvent_Order(this.field0): super._();


 final  WireUpbitOrderStreamEvent field0;

/// Create a copy of WireUpbitAccountStreamEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireUpbitAccountStreamEvent_OrderCopyWith<WireUpbitAccountStreamEvent_Order> get copyWith => _$WireUpbitAccountStreamEvent_OrderCopyWithImpl<WireUpbitAccountStreamEvent_Order>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitAccountStreamEvent_Order&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireUpbitAccountStreamEvent.order(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireUpbitAccountStreamEvent_OrderCopyWith<$Res> implements $WireUpbitAccountStreamEventCopyWith<$Res> {
  factory $WireUpbitAccountStreamEvent_OrderCopyWith(WireUpbitAccountStreamEvent_Order value, $Res Function(WireUpbitAccountStreamEvent_Order) _then) = _$WireUpbitAccountStreamEvent_OrderCopyWithImpl;
@useResult
$Res call({
 WireUpbitOrderStreamEvent field0
});




}
/// @nodoc
class _$WireUpbitAccountStreamEvent_OrderCopyWithImpl<$Res>
    implements $WireUpbitAccountStreamEvent_OrderCopyWith<$Res> {
  _$WireUpbitAccountStreamEvent_OrderCopyWithImpl(this._self, this._then);

  final WireUpbitAccountStreamEvent_Order _self;
  final $Res Function(WireUpbitAccountStreamEvent_Order) _then;

/// Create a copy of WireUpbitAccountStreamEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireUpbitAccountStreamEvent_Order(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireUpbitOrderStreamEvent,
  ));
}


}

/// @nodoc


class WireUpbitAccountStreamEvent_Reconnected extends WireUpbitAccountStreamEvent {
  const WireUpbitAccountStreamEvent_Reconnected(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitAccountStreamEvent_Reconnected);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireUpbitAccountStreamEvent.reconnected()';
}


}




/// @nodoc
mixin _$WireUpbitBatchCancelScope {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitBatchCancelScope);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireUpbitBatchCancelScope()';
}


}

/// @nodoc
class $WireUpbitBatchCancelScopeCopyWith<$Res>  {
$WireUpbitBatchCancelScopeCopyWith(WireUpbitBatchCancelScope _, $Res Function(WireUpbitBatchCancelScope) __);
}


/// Adds pattern-matching-related methods to [WireUpbitBatchCancelScope].
extension WireUpbitBatchCancelScopePatterns on WireUpbitBatchCancelScope {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireUpbitBatchCancelScope_All value)?  all,TResult Function( WireUpbitBatchCancelScope_QuoteCurrencies value)?  quoteCurrencies,TResult Function( WireUpbitBatchCancelScope_Pairs value)?  pairs,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireUpbitBatchCancelScope_All() when all != null:
return all(_that);case WireUpbitBatchCancelScope_QuoteCurrencies() when quoteCurrencies != null:
return quoteCurrencies(_that);case WireUpbitBatchCancelScope_Pairs() when pairs != null:
return pairs(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireUpbitBatchCancelScope_All value)  all,required TResult Function( WireUpbitBatchCancelScope_QuoteCurrencies value)  quoteCurrencies,required TResult Function( WireUpbitBatchCancelScope_Pairs value)  pairs,}){
final _that = this;
switch (_that) {
case WireUpbitBatchCancelScope_All():
return all(_that);case WireUpbitBatchCancelScope_QuoteCurrencies():
return quoteCurrencies(_that);case WireUpbitBatchCancelScope_Pairs():
return pairs(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireUpbitBatchCancelScope_All value)?  all,TResult? Function( WireUpbitBatchCancelScope_QuoteCurrencies value)?  quoteCurrencies,TResult? Function( WireUpbitBatchCancelScope_Pairs value)?  pairs,}){
final _that = this;
switch (_that) {
case WireUpbitBatchCancelScope_All() when all != null:
return all(_that);case WireUpbitBatchCancelScope_QuoteCurrencies() when quoteCurrencies != null:
return quoteCurrencies(_that);case WireUpbitBatchCancelScope_Pairs() when pairs != null:
return pairs(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  all,TResult Function( List<String> values)?  quoteCurrencies,TResult Function( List<WireMarket> values)?  pairs,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireUpbitBatchCancelScope_All() when all != null:
return all();case WireUpbitBatchCancelScope_QuoteCurrencies() when quoteCurrencies != null:
return quoteCurrencies(_that.values);case WireUpbitBatchCancelScope_Pairs() when pairs != null:
return pairs(_that.values);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  all,required TResult Function( List<String> values)  quoteCurrencies,required TResult Function( List<WireMarket> values)  pairs,}) {final _that = this;
switch (_that) {
case WireUpbitBatchCancelScope_All():
return all();case WireUpbitBatchCancelScope_QuoteCurrencies():
return quoteCurrencies(_that.values);case WireUpbitBatchCancelScope_Pairs():
return pairs(_that.values);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  all,TResult? Function( List<String> values)?  quoteCurrencies,TResult? Function( List<WireMarket> values)?  pairs,}) {final _that = this;
switch (_that) {
case WireUpbitBatchCancelScope_All() when all != null:
return all();case WireUpbitBatchCancelScope_QuoteCurrencies() when quoteCurrencies != null:
return quoteCurrencies(_that.values);case WireUpbitBatchCancelScope_Pairs() when pairs != null:
return pairs(_that.values);case _:
  return null;

}
}

}

/// @nodoc


class WireUpbitBatchCancelScope_All extends WireUpbitBatchCancelScope {
  const WireUpbitBatchCancelScope_All(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitBatchCancelScope_All);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireUpbitBatchCancelScope.all()';
}


}




/// @nodoc


class WireUpbitBatchCancelScope_QuoteCurrencies extends WireUpbitBatchCancelScope {
  const WireUpbitBatchCancelScope_QuoteCurrencies({required final  List<String> values}): _values = values,super._();


 final  List<String> _values;
 List<String> get values {
  if (_values is EqualUnmodifiableListView) return _values;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_values);
}


/// Create a copy of WireUpbitBatchCancelScope
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireUpbitBatchCancelScope_QuoteCurrenciesCopyWith<WireUpbitBatchCancelScope_QuoteCurrencies> get copyWith => _$WireUpbitBatchCancelScope_QuoteCurrenciesCopyWithImpl<WireUpbitBatchCancelScope_QuoteCurrencies>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitBatchCancelScope_QuoteCurrencies&&const DeepCollectionEquality().equals(other._values, _values));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_values));

@override
String toString() {
  return 'WireUpbitBatchCancelScope.quoteCurrencies(values: $values)';
}


}

/// @nodoc
abstract mixin class $WireUpbitBatchCancelScope_QuoteCurrenciesCopyWith<$Res> implements $WireUpbitBatchCancelScopeCopyWith<$Res> {
  factory $WireUpbitBatchCancelScope_QuoteCurrenciesCopyWith(WireUpbitBatchCancelScope_QuoteCurrencies value, $Res Function(WireUpbitBatchCancelScope_QuoteCurrencies) _then) = _$WireUpbitBatchCancelScope_QuoteCurrenciesCopyWithImpl;
@useResult
$Res call({
 List<String> values
});




}
/// @nodoc
class _$WireUpbitBatchCancelScope_QuoteCurrenciesCopyWithImpl<$Res>
    implements $WireUpbitBatchCancelScope_QuoteCurrenciesCopyWith<$Res> {
  _$WireUpbitBatchCancelScope_QuoteCurrenciesCopyWithImpl(this._self, this._then);

  final WireUpbitBatchCancelScope_QuoteCurrencies _self;
  final $Res Function(WireUpbitBatchCancelScope_QuoteCurrencies) _then;

/// Create a copy of WireUpbitBatchCancelScope
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? values = null,}) {
  return _then(WireUpbitBatchCancelScope_QuoteCurrencies(
values: null == values ? _self._values : values // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}


}

/// @nodoc


class WireUpbitBatchCancelScope_Pairs extends WireUpbitBatchCancelScope {
  const WireUpbitBatchCancelScope_Pairs({required final  List<WireMarket> values}): _values = values,super._();


 final  List<WireMarket> _values;
 List<WireMarket> get values {
  if (_values is EqualUnmodifiableListView) return _values;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_values);
}


/// Create a copy of WireUpbitBatchCancelScope
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireUpbitBatchCancelScope_PairsCopyWith<WireUpbitBatchCancelScope_Pairs> get copyWith => _$WireUpbitBatchCancelScope_PairsCopyWithImpl<WireUpbitBatchCancelScope_Pairs>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitBatchCancelScope_Pairs&&const DeepCollectionEquality().equals(other._values, _values));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_values));

@override
String toString() {
  return 'WireUpbitBatchCancelScope.pairs(values: $values)';
}


}

/// @nodoc
abstract mixin class $WireUpbitBatchCancelScope_PairsCopyWith<$Res> implements $WireUpbitBatchCancelScopeCopyWith<$Res> {
  factory $WireUpbitBatchCancelScope_PairsCopyWith(WireUpbitBatchCancelScope_Pairs value, $Res Function(WireUpbitBatchCancelScope_Pairs) _then) = _$WireUpbitBatchCancelScope_PairsCopyWithImpl;
@useResult
$Res call({
 List<WireMarket> values
});




}
/// @nodoc
class _$WireUpbitBatchCancelScope_PairsCopyWithImpl<$Res>
    implements $WireUpbitBatchCancelScope_PairsCopyWith<$Res> {
  _$WireUpbitBatchCancelScope_PairsCopyWithImpl(this._self, this._then);

  final WireUpbitBatchCancelScope_Pairs _self;
  final $Res Function(WireUpbitBatchCancelScope_Pairs) _then;

/// Create a copy of WireUpbitBatchCancelScope
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? values = null,}) {
  return _then(WireUpbitBatchCancelScope_Pairs(
values: null == values ? _self._values : values // ignore: cast_nullable_to_non_nullable
as List<WireMarket>,
  ));
}


}

/// @nodoc
mixin _$WireUpbitCancelAndNewOrder {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitCancelAndNewOrder);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireUpbitCancelAndNewOrder()';
}


}

/// @nodoc
class $WireUpbitCancelAndNewOrderCopyWith<$Res>  {
$WireUpbitCancelAndNewOrderCopyWith(WireUpbitCancelAndNewOrder _, $Res Function(WireUpbitCancelAndNewOrder) __);
}


/// Adds pattern-matching-related methods to [WireUpbitCancelAndNewOrder].
extension WireUpbitCancelAndNewOrderPatterns on WireUpbitCancelAndNewOrder {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireUpbitCancelAndNewOrder_Limit value)?  limit,TResult Function( WireUpbitCancelAndNewOrder_MarketBuy value)?  marketBuy,TResult Function( WireUpbitCancelAndNewOrder_MarketSell value)?  marketSell,TResult Function( WireUpbitCancelAndNewOrder_BestBuy value)?  bestBuy,TResult Function( WireUpbitCancelAndNewOrder_BestSell value)?  bestSell,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireUpbitCancelAndNewOrder_Limit() when limit != null:
return limit(_that);case WireUpbitCancelAndNewOrder_MarketBuy() when marketBuy != null:
return marketBuy(_that);case WireUpbitCancelAndNewOrder_MarketSell() when marketSell != null:
return marketSell(_that);case WireUpbitCancelAndNewOrder_BestBuy() when bestBuy != null:
return bestBuy(_that);case WireUpbitCancelAndNewOrder_BestSell() when bestSell != null:
return bestSell(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireUpbitCancelAndNewOrder_Limit value)  limit,required TResult Function( WireUpbitCancelAndNewOrder_MarketBuy value)  marketBuy,required TResult Function( WireUpbitCancelAndNewOrder_MarketSell value)  marketSell,required TResult Function( WireUpbitCancelAndNewOrder_BestBuy value)  bestBuy,required TResult Function( WireUpbitCancelAndNewOrder_BestSell value)  bestSell,}){
final _that = this;
switch (_that) {
case WireUpbitCancelAndNewOrder_Limit():
return limit(_that);case WireUpbitCancelAndNewOrder_MarketBuy():
return marketBuy(_that);case WireUpbitCancelAndNewOrder_MarketSell():
return marketSell(_that);case WireUpbitCancelAndNewOrder_BestBuy():
return bestBuy(_that);case WireUpbitCancelAndNewOrder_BestSell():
return bestSell(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireUpbitCancelAndNewOrder_Limit value)?  limit,TResult? Function( WireUpbitCancelAndNewOrder_MarketBuy value)?  marketBuy,TResult? Function( WireUpbitCancelAndNewOrder_MarketSell value)?  marketSell,TResult? Function( WireUpbitCancelAndNewOrder_BestBuy value)?  bestBuy,TResult? Function( WireUpbitCancelAndNewOrder_BestSell value)?  bestSell,}){
final _that = this;
switch (_that) {
case WireUpbitCancelAndNewOrder_Limit() when limit != null:
return limit(_that);case WireUpbitCancelAndNewOrder_MarketBuy() when marketBuy != null:
return marketBuy(_that);case WireUpbitCancelAndNewOrder_MarketSell() when marketSell != null:
return marketSell(_that);case WireUpbitCancelAndNewOrder_BestBuy() when bestBuy != null:
return bestBuy(_that);case WireUpbitCancelAndNewOrder_BestSell() when bestSell != null:
return bestSell(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( WireUpbitOrderVolume volume,  String price,  WireTimeInForce? timeInForce)?  limit,TResult Function( String price)?  marketBuy,TResult Function( WireUpbitOrderVolume volume)?  marketSell,TResult Function( String price,  WireTimeInForce timeInForce)?  bestBuy,TResult Function( WireUpbitOrderVolume volume,  WireTimeInForce timeInForce)?  bestSell,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireUpbitCancelAndNewOrder_Limit() when limit != null:
return limit(_that.volume,_that.price,_that.timeInForce);case WireUpbitCancelAndNewOrder_MarketBuy() when marketBuy != null:
return marketBuy(_that.price);case WireUpbitCancelAndNewOrder_MarketSell() when marketSell != null:
return marketSell(_that.volume);case WireUpbitCancelAndNewOrder_BestBuy() when bestBuy != null:
return bestBuy(_that.price,_that.timeInForce);case WireUpbitCancelAndNewOrder_BestSell() when bestSell != null:
return bestSell(_that.volume,_that.timeInForce);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( WireUpbitOrderVolume volume,  String price,  WireTimeInForce? timeInForce)  limit,required TResult Function( String price)  marketBuy,required TResult Function( WireUpbitOrderVolume volume)  marketSell,required TResult Function( String price,  WireTimeInForce timeInForce)  bestBuy,required TResult Function( WireUpbitOrderVolume volume,  WireTimeInForce timeInForce)  bestSell,}) {final _that = this;
switch (_that) {
case WireUpbitCancelAndNewOrder_Limit():
return limit(_that.volume,_that.price,_that.timeInForce);case WireUpbitCancelAndNewOrder_MarketBuy():
return marketBuy(_that.price);case WireUpbitCancelAndNewOrder_MarketSell():
return marketSell(_that.volume);case WireUpbitCancelAndNewOrder_BestBuy():
return bestBuy(_that.price,_that.timeInForce);case WireUpbitCancelAndNewOrder_BestSell():
return bestSell(_that.volume,_that.timeInForce);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( WireUpbitOrderVolume volume,  String price,  WireTimeInForce? timeInForce)?  limit,TResult? Function( String price)?  marketBuy,TResult? Function( WireUpbitOrderVolume volume)?  marketSell,TResult? Function( String price,  WireTimeInForce timeInForce)?  bestBuy,TResult? Function( WireUpbitOrderVolume volume,  WireTimeInForce timeInForce)?  bestSell,}) {final _that = this;
switch (_that) {
case WireUpbitCancelAndNewOrder_Limit() when limit != null:
return limit(_that.volume,_that.price,_that.timeInForce);case WireUpbitCancelAndNewOrder_MarketBuy() when marketBuy != null:
return marketBuy(_that.price);case WireUpbitCancelAndNewOrder_MarketSell() when marketSell != null:
return marketSell(_that.volume);case WireUpbitCancelAndNewOrder_BestBuy() when bestBuy != null:
return bestBuy(_that.price,_that.timeInForce);case WireUpbitCancelAndNewOrder_BestSell() when bestSell != null:
return bestSell(_that.volume,_that.timeInForce);case _:
  return null;

}
}

}

/// @nodoc


class WireUpbitCancelAndNewOrder_Limit extends WireUpbitCancelAndNewOrder {
  const WireUpbitCancelAndNewOrder_Limit({required this.volume, required this.price, this.timeInForce}): super._();


 final  WireUpbitOrderVolume volume;
 final  String price;
 final  WireTimeInForce? timeInForce;

/// Create a copy of WireUpbitCancelAndNewOrder
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireUpbitCancelAndNewOrder_LimitCopyWith<WireUpbitCancelAndNewOrder_Limit> get copyWith => _$WireUpbitCancelAndNewOrder_LimitCopyWithImpl<WireUpbitCancelAndNewOrder_Limit>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitCancelAndNewOrder_Limit&&(identical(other.volume, volume) || other.volume == volume)&&(identical(other.price, price) || other.price == price)&&(identical(other.timeInForce, timeInForce) || other.timeInForce == timeInForce));
}


@override
int get hashCode => Object.hash(runtimeType,volume,price,timeInForce);

@override
String toString() {
  return 'WireUpbitCancelAndNewOrder.limit(volume: $volume, price: $price, timeInForce: $timeInForce)';
}


}

/// @nodoc
abstract mixin class $WireUpbitCancelAndNewOrder_LimitCopyWith<$Res> implements $WireUpbitCancelAndNewOrderCopyWith<$Res> {
  factory $WireUpbitCancelAndNewOrder_LimitCopyWith(WireUpbitCancelAndNewOrder_Limit value, $Res Function(WireUpbitCancelAndNewOrder_Limit) _then) = _$WireUpbitCancelAndNewOrder_LimitCopyWithImpl;
@useResult
$Res call({
 WireUpbitOrderVolume volume, String price, WireTimeInForce? timeInForce
});


$WireUpbitOrderVolumeCopyWith<$Res> get volume;

}
/// @nodoc
class _$WireUpbitCancelAndNewOrder_LimitCopyWithImpl<$Res>
    implements $WireUpbitCancelAndNewOrder_LimitCopyWith<$Res> {
  _$WireUpbitCancelAndNewOrder_LimitCopyWithImpl(this._self, this._then);

  final WireUpbitCancelAndNewOrder_Limit _self;
  final $Res Function(WireUpbitCancelAndNewOrder_Limit) _then;

/// Create a copy of WireUpbitCancelAndNewOrder
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? volume = null,Object? price = null,Object? timeInForce = freezed,}) {
  return _then(WireUpbitCancelAndNewOrder_Limit(
volume: null == volume ? _self.volume : volume // ignore: cast_nullable_to_non_nullable
as WireUpbitOrderVolume,price: null == price ? _self.price : price // ignore: cast_nullable_to_non_nullable
as String,timeInForce: freezed == timeInForce ? _self.timeInForce : timeInForce // ignore: cast_nullable_to_non_nullable
as WireTimeInForce?,
  ));
}

/// Create a copy of WireUpbitCancelAndNewOrder
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$WireUpbitOrderVolumeCopyWith<$Res> get volume {

  return $WireUpbitOrderVolumeCopyWith<$Res>(_self.volume, (value) {
    return _then(_self.copyWith(volume: value));
  });
}
}

/// @nodoc


class WireUpbitCancelAndNewOrder_MarketBuy extends WireUpbitCancelAndNewOrder {
  const WireUpbitCancelAndNewOrder_MarketBuy({required this.price}): super._();


 final  String price;

/// Create a copy of WireUpbitCancelAndNewOrder
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireUpbitCancelAndNewOrder_MarketBuyCopyWith<WireUpbitCancelAndNewOrder_MarketBuy> get copyWith => _$WireUpbitCancelAndNewOrder_MarketBuyCopyWithImpl<WireUpbitCancelAndNewOrder_MarketBuy>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitCancelAndNewOrder_MarketBuy&&(identical(other.price, price) || other.price == price));
}


@override
int get hashCode => Object.hash(runtimeType,price);

@override
String toString() {
  return 'WireUpbitCancelAndNewOrder.marketBuy(price: $price)';
}


}

/// @nodoc
abstract mixin class $WireUpbitCancelAndNewOrder_MarketBuyCopyWith<$Res> implements $WireUpbitCancelAndNewOrderCopyWith<$Res> {
  factory $WireUpbitCancelAndNewOrder_MarketBuyCopyWith(WireUpbitCancelAndNewOrder_MarketBuy value, $Res Function(WireUpbitCancelAndNewOrder_MarketBuy) _then) = _$WireUpbitCancelAndNewOrder_MarketBuyCopyWithImpl;
@useResult
$Res call({
 String price
});




}
/// @nodoc
class _$WireUpbitCancelAndNewOrder_MarketBuyCopyWithImpl<$Res>
    implements $WireUpbitCancelAndNewOrder_MarketBuyCopyWith<$Res> {
  _$WireUpbitCancelAndNewOrder_MarketBuyCopyWithImpl(this._self, this._then);

  final WireUpbitCancelAndNewOrder_MarketBuy _self;
  final $Res Function(WireUpbitCancelAndNewOrder_MarketBuy) _then;

/// Create a copy of WireUpbitCancelAndNewOrder
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? price = null,}) {
  return _then(WireUpbitCancelAndNewOrder_MarketBuy(
price: null == price ? _self.price : price // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class WireUpbitCancelAndNewOrder_MarketSell extends WireUpbitCancelAndNewOrder {
  const WireUpbitCancelAndNewOrder_MarketSell({required this.volume}): super._();


 final  WireUpbitOrderVolume volume;

/// Create a copy of WireUpbitCancelAndNewOrder
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireUpbitCancelAndNewOrder_MarketSellCopyWith<WireUpbitCancelAndNewOrder_MarketSell> get copyWith => _$WireUpbitCancelAndNewOrder_MarketSellCopyWithImpl<WireUpbitCancelAndNewOrder_MarketSell>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitCancelAndNewOrder_MarketSell&&(identical(other.volume, volume) || other.volume == volume));
}


@override
int get hashCode => Object.hash(runtimeType,volume);

@override
String toString() {
  return 'WireUpbitCancelAndNewOrder.marketSell(volume: $volume)';
}


}

/// @nodoc
abstract mixin class $WireUpbitCancelAndNewOrder_MarketSellCopyWith<$Res> implements $WireUpbitCancelAndNewOrderCopyWith<$Res> {
  factory $WireUpbitCancelAndNewOrder_MarketSellCopyWith(WireUpbitCancelAndNewOrder_MarketSell value, $Res Function(WireUpbitCancelAndNewOrder_MarketSell) _then) = _$WireUpbitCancelAndNewOrder_MarketSellCopyWithImpl;
@useResult
$Res call({
 WireUpbitOrderVolume volume
});


$WireUpbitOrderVolumeCopyWith<$Res> get volume;

}
/// @nodoc
class _$WireUpbitCancelAndNewOrder_MarketSellCopyWithImpl<$Res>
    implements $WireUpbitCancelAndNewOrder_MarketSellCopyWith<$Res> {
  _$WireUpbitCancelAndNewOrder_MarketSellCopyWithImpl(this._self, this._then);

  final WireUpbitCancelAndNewOrder_MarketSell _self;
  final $Res Function(WireUpbitCancelAndNewOrder_MarketSell) _then;

/// Create a copy of WireUpbitCancelAndNewOrder
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? volume = null,}) {
  return _then(WireUpbitCancelAndNewOrder_MarketSell(
volume: null == volume ? _self.volume : volume // ignore: cast_nullable_to_non_nullable
as WireUpbitOrderVolume,
  ));
}

/// Create a copy of WireUpbitCancelAndNewOrder
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$WireUpbitOrderVolumeCopyWith<$Res> get volume {

  return $WireUpbitOrderVolumeCopyWith<$Res>(_self.volume, (value) {
    return _then(_self.copyWith(volume: value));
  });
}
}

/// @nodoc


class WireUpbitCancelAndNewOrder_BestBuy extends WireUpbitCancelAndNewOrder {
  const WireUpbitCancelAndNewOrder_BestBuy({required this.price, required this.timeInForce}): super._();


 final  String price;
 final  WireTimeInForce timeInForce;

/// Create a copy of WireUpbitCancelAndNewOrder
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireUpbitCancelAndNewOrder_BestBuyCopyWith<WireUpbitCancelAndNewOrder_BestBuy> get copyWith => _$WireUpbitCancelAndNewOrder_BestBuyCopyWithImpl<WireUpbitCancelAndNewOrder_BestBuy>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitCancelAndNewOrder_BestBuy&&(identical(other.price, price) || other.price == price)&&(identical(other.timeInForce, timeInForce) || other.timeInForce == timeInForce));
}


@override
int get hashCode => Object.hash(runtimeType,price,timeInForce);

@override
String toString() {
  return 'WireUpbitCancelAndNewOrder.bestBuy(price: $price, timeInForce: $timeInForce)';
}


}

/// @nodoc
abstract mixin class $WireUpbitCancelAndNewOrder_BestBuyCopyWith<$Res> implements $WireUpbitCancelAndNewOrderCopyWith<$Res> {
  factory $WireUpbitCancelAndNewOrder_BestBuyCopyWith(WireUpbitCancelAndNewOrder_BestBuy value, $Res Function(WireUpbitCancelAndNewOrder_BestBuy) _then) = _$WireUpbitCancelAndNewOrder_BestBuyCopyWithImpl;
@useResult
$Res call({
 String price, WireTimeInForce timeInForce
});




}
/// @nodoc
class _$WireUpbitCancelAndNewOrder_BestBuyCopyWithImpl<$Res>
    implements $WireUpbitCancelAndNewOrder_BestBuyCopyWith<$Res> {
  _$WireUpbitCancelAndNewOrder_BestBuyCopyWithImpl(this._self, this._then);

  final WireUpbitCancelAndNewOrder_BestBuy _self;
  final $Res Function(WireUpbitCancelAndNewOrder_BestBuy) _then;

/// Create a copy of WireUpbitCancelAndNewOrder
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? price = null,Object? timeInForce = null,}) {
  return _then(WireUpbitCancelAndNewOrder_BestBuy(
price: null == price ? _self.price : price // ignore: cast_nullable_to_non_nullable
as String,timeInForce: null == timeInForce ? _self.timeInForce : timeInForce // ignore: cast_nullable_to_non_nullable
as WireTimeInForce,
  ));
}


}

/// @nodoc


class WireUpbitCancelAndNewOrder_BestSell extends WireUpbitCancelAndNewOrder {
  const WireUpbitCancelAndNewOrder_BestSell({required this.volume, required this.timeInForce}): super._();


 final  WireUpbitOrderVolume volume;
 final  WireTimeInForce timeInForce;

/// Create a copy of WireUpbitCancelAndNewOrder
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireUpbitCancelAndNewOrder_BestSellCopyWith<WireUpbitCancelAndNewOrder_BestSell> get copyWith => _$WireUpbitCancelAndNewOrder_BestSellCopyWithImpl<WireUpbitCancelAndNewOrder_BestSell>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitCancelAndNewOrder_BestSell&&(identical(other.volume, volume) || other.volume == volume)&&(identical(other.timeInForce, timeInForce) || other.timeInForce == timeInForce));
}


@override
int get hashCode => Object.hash(runtimeType,volume,timeInForce);

@override
String toString() {
  return 'WireUpbitCancelAndNewOrder.bestSell(volume: $volume, timeInForce: $timeInForce)';
}


}

/// @nodoc
abstract mixin class $WireUpbitCancelAndNewOrder_BestSellCopyWith<$Res> implements $WireUpbitCancelAndNewOrderCopyWith<$Res> {
  factory $WireUpbitCancelAndNewOrder_BestSellCopyWith(WireUpbitCancelAndNewOrder_BestSell value, $Res Function(WireUpbitCancelAndNewOrder_BestSell) _then) = _$WireUpbitCancelAndNewOrder_BestSellCopyWithImpl;
@useResult
$Res call({
 WireUpbitOrderVolume volume, WireTimeInForce timeInForce
});


$WireUpbitOrderVolumeCopyWith<$Res> get volume;

}
/// @nodoc
class _$WireUpbitCancelAndNewOrder_BestSellCopyWithImpl<$Res>
    implements $WireUpbitCancelAndNewOrder_BestSellCopyWith<$Res> {
  _$WireUpbitCancelAndNewOrder_BestSellCopyWithImpl(this._self, this._then);

  final WireUpbitCancelAndNewOrder_BestSell _self;
  final $Res Function(WireUpbitCancelAndNewOrder_BestSell) _then;

/// Create a copy of WireUpbitCancelAndNewOrder
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? volume = null,Object? timeInForce = null,}) {
  return _then(WireUpbitCancelAndNewOrder_BestSell(
volume: null == volume ? _self.volume : volume // ignore: cast_nullable_to_non_nullable
as WireUpbitOrderVolume,timeInForce: null == timeInForce ? _self.timeInForce : timeInForce // ignore: cast_nullable_to_non_nullable
as WireTimeInForce,
  ));
}

/// Create a copy of WireUpbitCancelAndNewOrder
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$WireUpbitOrderVolumeCopyWith<$Res> get volume {

  return $WireUpbitOrderVolumeCopyWith<$Res>(_self.volume, (value) {
    return _then(_self.copyWith(volume: value));
  });
}
}

/// @nodoc
mixin _$WireUpbitMarketStreamEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitMarketStreamEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireUpbitMarketStreamEvent()';
}


}

/// @nodoc
class $WireUpbitMarketStreamEventCopyWith<$Res>  {
$WireUpbitMarketStreamEventCopyWith(WireUpbitMarketStreamEvent _, $Res Function(WireUpbitMarketStreamEvent) __);
}


/// Adds pattern-matching-related methods to [WireUpbitMarketStreamEvent].
extension WireUpbitMarketStreamEventPatterns on WireUpbitMarketStreamEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireUpbitMarketStreamEvent_Trade value)?  trade,TResult Function( WireUpbitMarketStreamEvent_OrderBook value)?  orderBook,TResult Function( WireUpbitMarketStreamEvent_Ticker value)?  ticker,TResult Function( WireUpbitMarketStreamEvent_Candle value)?  candle,TResult Function( WireUpbitMarketStreamEvent_Reconnected value)?  reconnected,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireUpbitMarketStreamEvent_Trade() when trade != null:
return trade(_that);case WireUpbitMarketStreamEvent_OrderBook() when orderBook != null:
return orderBook(_that);case WireUpbitMarketStreamEvent_Ticker() when ticker != null:
return ticker(_that);case WireUpbitMarketStreamEvent_Candle() when candle != null:
return candle(_that);case WireUpbitMarketStreamEvent_Reconnected() when reconnected != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireUpbitMarketStreamEvent_Trade value)  trade,required TResult Function( WireUpbitMarketStreamEvent_OrderBook value)  orderBook,required TResult Function( WireUpbitMarketStreamEvent_Ticker value)  ticker,required TResult Function( WireUpbitMarketStreamEvent_Candle value)  candle,required TResult Function( WireUpbitMarketStreamEvent_Reconnected value)  reconnected,}){
final _that = this;
switch (_that) {
case WireUpbitMarketStreamEvent_Trade():
return trade(_that);case WireUpbitMarketStreamEvent_OrderBook():
return orderBook(_that);case WireUpbitMarketStreamEvent_Ticker():
return ticker(_that);case WireUpbitMarketStreamEvent_Candle():
return candle(_that);case WireUpbitMarketStreamEvent_Reconnected():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireUpbitMarketStreamEvent_Trade value)?  trade,TResult? Function( WireUpbitMarketStreamEvent_OrderBook value)?  orderBook,TResult? Function( WireUpbitMarketStreamEvent_Ticker value)?  ticker,TResult? Function( WireUpbitMarketStreamEvent_Candle value)?  candle,TResult? Function( WireUpbitMarketStreamEvent_Reconnected value)?  reconnected,}){
final _that = this;
switch (_that) {
case WireUpbitMarketStreamEvent_Trade() when trade != null:
return trade(_that);case WireUpbitMarketStreamEvent_OrderBook() when orderBook != null:
return orderBook(_that);case WireUpbitMarketStreamEvent_Ticker() when ticker != null:
return ticker(_that);case WireUpbitMarketStreamEvent_Candle() when candle != null:
return candle(_that);case WireUpbitMarketStreamEvent_Reconnected() when reconnected != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( WireUpbitTradeStreamEvent field0)?  trade,TResult Function( WireUpbitOrderBookStreamEvent field0)?  orderBook,TResult Function( WireUpbitTickerStreamEvent field0)?  ticker,TResult Function( WireUpbitCandleStreamEvent field0)?  candle,TResult Function()?  reconnected,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireUpbitMarketStreamEvent_Trade() when trade != null:
return trade(_that.field0);case WireUpbitMarketStreamEvent_OrderBook() when orderBook != null:
return orderBook(_that.field0);case WireUpbitMarketStreamEvent_Ticker() when ticker != null:
return ticker(_that.field0);case WireUpbitMarketStreamEvent_Candle() when candle != null:
return candle(_that.field0);case WireUpbitMarketStreamEvent_Reconnected() when reconnected != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( WireUpbitTradeStreamEvent field0)  trade,required TResult Function( WireUpbitOrderBookStreamEvent field0)  orderBook,required TResult Function( WireUpbitTickerStreamEvent field0)  ticker,required TResult Function( WireUpbitCandleStreamEvent field0)  candle,required TResult Function()  reconnected,}) {final _that = this;
switch (_that) {
case WireUpbitMarketStreamEvent_Trade():
return trade(_that.field0);case WireUpbitMarketStreamEvent_OrderBook():
return orderBook(_that.field0);case WireUpbitMarketStreamEvent_Ticker():
return ticker(_that.field0);case WireUpbitMarketStreamEvent_Candle():
return candle(_that.field0);case WireUpbitMarketStreamEvent_Reconnected():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( WireUpbitTradeStreamEvent field0)?  trade,TResult? Function( WireUpbitOrderBookStreamEvent field0)?  orderBook,TResult? Function( WireUpbitTickerStreamEvent field0)?  ticker,TResult? Function( WireUpbitCandleStreamEvent field0)?  candle,TResult? Function()?  reconnected,}) {final _that = this;
switch (_that) {
case WireUpbitMarketStreamEvent_Trade() when trade != null:
return trade(_that.field0);case WireUpbitMarketStreamEvent_OrderBook() when orderBook != null:
return orderBook(_that.field0);case WireUpbitMarketStreamEvent_Ticker() when ticker != null:
return ticker(_that.field0);case WireUpbitMarketStreamEvent_Candle() when candle != null:
return candle(_that.field0);case WireUpbitMarketStreamEvent_Reconnected() when reconnected != null:
return reconnected();case _:
  return null;

}
}

}

/// @nodoc


class WireUpbitMarketStreamEvent_Trade extends WireUpbitMarketStreamEvent {
  const WireUpbitMarketStreamEvent_Trade(this.field0): super._();


 final  WireUpbitTradeStreamEvent field0;

/// Create a copy of WireUpbitMarketStreamEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireUpbitMarketStreamEvent_TradeCopyWith<WireUpbitMarketStreamEvent_Trade> get copyWith => _$WireUpbitMarketStreamEvent_TradeCopyWithImpl<WireUpbitMarketStreamEvent_Trade>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitMarketStreamEvent_Trade&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireUpbitMarketStreamEvent.trade(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireUpbitMarketStreamEvent_TradeCopyWith<$Res> implements $WireUpbitMarketStreamEventCopyWith<$Res> {
  factory $WireUpbitMarketStreamEvent_TradeCopyWith(WireUpbitMarketStreamEvent_Trade value, $Res Function(WireUpbitMarketStreamEvent_Trade) _then) = _$WireUpbitMarketStreamEvent_TradeCopyWithImpl;
@useResult
$Res call({
 WireUpbitTradeStreamEvent field0
});




}
/// @nodoc
class _$WireUpbitMarketStreamEvent_TradeCopyWithImpl<$Res>
    implements $WireUpbitMarketStreamEvent_TradeCopyWith<$Res> {
  _$WireUpbitMarketStreamEvent_TradeCopyWithImpl(this._self, this._then);

  final WireUpbitMarketStreamEvent_Trade _self;
  final $Res Function(WireUpbitMarketStreamEvent_Trade) _then;

/// Create a copy of WireUpbitMarketStreamEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireUpbitMarketStreamEvent_Trade(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireUpbitTradeStreamEvent,
  ));
}


}

/// @nodoc


class WireUpbitMarketStreamEvent_OrderBook extends WireUpbitMarketStreamEvent {
  const WireUpbitMarketStreamEvent_OrderBook(this.field0): super._();


 final  WireUpbitOrderBookStreamEvent field0;

/// Create a copy of WireUpbitMarketStreamEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireUpbitMarketStreamEvent_OrderBookCopyWith<WireUpbitMarketStreamEvent_OrderBook> get copyWith => _$WireUpbitMarketStreamEvent_OrderBookCopyWithImpl<WireUpbitMarketStreamEvent_OrderBook>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitMarketStreamEvent_OrderBook&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireUpbitMarketStreamEvent.orderBook(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireUpbitMarketStreamEvent_OrderBookCopyWith<$Res> implements $WireUpbitMarketStreamEventCopyWith<$Res> {
  factory $WireUpbitMarketStreamEvent_OrderBookCopyWith(WireUpbitMarketStreamEvent_OrderBook value, $Res Function(WireUpbitMarketStreamEvent_OrderBook) _then) = _$WireUpbitMarketStreamEvent_OrderBookCopyWithImpl;
@useResult
$Res call({
 WireUpbitOrderBookStreamEvent field0
});




}
/// @nodoc
class _$WireUpbitMarketStreamEvent_OrderBookCopyWithImpl<$Res>
    implements $WireUpbitMarketStreamEvent_OrderBookCopyWith<$Res> {
  _$WireUpbitMarketStreamEvent_OrderBookCopyWithImpl(this._self, this._then);

  final WireUpbitMarketStreamEvent_OrderBook _self;
  final $Res Function(WireUpbitMarketStreamEvent_OrderBook) _then;

/// Create a copy of WireUpbitMarketStreamEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireUpbitMarketStreamEvent_OrderBook(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireUpbitOrderBookStreamEvent,
  ));
}


}

/// @nodoc


class WireUpbitMarketStreamEvent_Ticker extends WireUpbitMarketStreamEvent {
  const WireUpbitMarketStreamEvent_Ticker(this.field0): super._();


 final  WireUpbitTickerStreamEvent field0;

/// Create a copy of WireUpbitMarketStreamEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireUpbitMarketStreamEvent_TickerCopyWith<WireUpbitMarketStreamEvent_Ticker> get copyWith => _$WireUpbitMarketStreamEvent_TickerCopyWithImpl<WireUpbitMarketStreamEvent_Ticker>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitMarketStreamEvent_Ticker&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireUpbitMarketStreamEvent.ticker(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireUpbitMarketStreamEvent_TickerCopyWith<$Res> implements $WireUpbitMarketStreamEventCopyWith<$Res> {
  factory $WireUpbitMarketStreamEvent_TickerCopyWith(WireUpbitMarketStreamEvent_Ticker value, $Res Function(WireUpbitMarketStreamEvent_Ticker) _then) = _$WireUpbitMarketStreamEvent_TickerCopyWithImpl;
@useResult
$Res call({
 WireUpbitTickerStreamEvent field0
});




}
/// @nodoc
class _$WireUpbitMarketStreamEvent_TickerCopyWithImpl<$Res>
    implements $WireUpbitMarketStreamEvent_TickerCopyWith<$Res> {
  _$WireUpbitMarketStreamEvent_TickerCopyWithImpl(this._self, this._then);

  final WireUpbitMarketStreamEvent_Ticker _self;
  final $Res Function(WireUpbitMarketStreamEvent_Ticker) _then;

/// Create a copy of WireUpbitMarketStreamEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireUpbitMarketStreamEvent_Ticker(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireUpbitTickerStreamEvent,
  ));
}


}

/// @nodoc


class WireUpbitMarketStreamEvent_Candle extends WireUpbitMarketStreamEvent {
  const WireUpbitMarketStreamEvent_Candle(this.field0): super._();


 final  WireUpbitCandleStreamEvent field0;

/// Create a copy of WireUpbitMarketStreamEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireUpbitMarketStreamEvent_CandleCopyWith<WireUpbitMarketStreamEvent_Candle> get copyWith => _$WireUpbitMarketStreamEvent_CandleCopyWithImpl<WireUpbitMarketStreamEvent_Candle>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitMarketStreamEvent_Candle&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireUpbitMarketStreamEvent.candle(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireUpbitMarketStreamEvent_CandleCopyWith<$Res> implements $WireUpbitMarketStreamEventCopyWith<$Res> {
  factory $WireUpbitMarketStreamEvent_CandleCopyWith(WireUpbitMarketStreamEvent_Candle value, $Res Function(WireUpbitMarketStreamEvent_Candle) _then) = _$WireUpbitMarketStreamEvent_CandleCopyWithImpl;
@useResult
$Res call({
 WireUpbitCandleStreamEvent field0
});




}
/// @nodoc
class _$WireUpbitMarketStreamEvent_CandleCopyWithImpl<$Res>
    implements $WireUpbitMarketStreamEvent_CandleCopyWith<$Res> {
  _$WireUpbitMarketStreamEvent_CandleCopyWithImpl(this._self, this._then);

  final WireUpbitMarketStreamEvent_Candle _self;
  final $Res Function(WireUpbitMarketStreamEvent_Candle) _then;

/// Create a copy of WireUpbitMarketStreamEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireUpbitMarketStreamEvent_Candle(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireUpbitCandleStreamEvent,
  ));
}


}

/// @nodoc


class WireUpbitMarketStreamEvent_Reconnected extends WireUpbitMarketStreamEvent {
  const WireUpbitMarketStreamEvent_Reconnected(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitMarketStreamEvent_Reconnected);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireUpbitMarketStreamEvent.reconnected()';
}


}




/// @nodoc
mixin _$WireUpbitOrderReference {

 String get field0;
/// Create a copy of WireUpbitOrderReference
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireUpbitOrderReferenceCopyWith<WireUpbitOrderReference> get copyWith => _$WireUpbitOrderReferenceCopyWithImpl<WireUpbitOrderReference>(this as WireUpbitOrderReference, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitOrderReference&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireUpbitOrderReference(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireUpbitOrderReferenceCopyWith<$Res>  {
  factory $WireUpbitOrderReferenceCopyWith(WireUpbitOrderReference value, $Res Function(WireUpbitOrderReference) _then) = _$WireUpbitOrderReferenceCopyWithImpl;
@useResult
$Res call({
 String field0
});




}
/// @nodoc
class _$WireUpbitOrderReferenceCopyWithImpl<$Res>
    implements $WireUpbitOrderReferenceCopyWith<$Res> {
  _$WireUpbitOrderReferenceCopyWithImpl(this._self, this._then);

  final WireUpbitOrderReference _self;
  final $Res Function(WireUpbitOrderReference) _then;

/// Create a copy of WireUpbitOrderReference
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? field0 = null,}) {
  return _then(_self.copyWith(
field0: null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [WireUpbitOrderReference].
extension WireUpbitOrderReferencePatterns on WireUpbitOrderReference {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireUpbitOrderReference_Uuid value)?  uuid,TResult Function( WireUpbitOrderReference_Identifier value)?  identifier,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireUpbitOrderReference_Uuid() when uuid != null:
return uuid(_that);case WireUpbitOrderReference_Identifier() when identifier != null:
return identifier(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireUpbitOrderReference_Uuid value)  uuid,required TResult Function( WireUpbitOrderReference_Identifier value)  identifier,}){
final _that = this;
switch (_that) {
case WireUpbitOrderReference_Uuid():
return uuid(_that);case WireUpbitOrderReference_Identifier():
return identifier(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireUpbitOrderReference_Uuid value)?  uuid,TResult? Function( WireUpbitOrderReference_Identifier value)?  identifier,}){
final _that = this;
switch (_that) {
case WireUpbitOrderReference_Uuid() when uuid != null:
return uuid(_that);case WireUpbitOrderReference_Identifier() when identifier != null:
return identifier(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String field0)?  uuid,TResult Function( String field0)?  identifier,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireUpbitOrderReference_Uuid() when uuid != null:
return uuid(_that.field0);case WireUpbitOrderReference_Identifier() when identifier != null:
return identifier(_that.field0);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String field0)  uuid,required TResult Function( String field0)  identifier,}) {final _that = this;
switch (_that) {
case WireUpbitOrderReference_Uuid():
return uuid(_that.field0);case WireUpbitOrderReference_Identifier():
return identifier(_that.field0);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String field0)?  uuid,TResult? Function( String field0)?  identifier,}) {final _that = this;
switch (_that) {
case WireUpbitOrderReference_Uuid() when uuid != null:
return uuid(_that.field0);case WireUpbitOrderReference_Identifier() when identifier != null:
return identifier(_that.field0);case _:
  return null;

}
}

}

/// @nodoc


class WireUpbitOrderReference_Uuid extends WireUpbitOrderReference {
  const WireUpbitOrderReference_Uuid(this.field0): super._();


@override final  String field0;

/// Create a copy of WireUpbitOrderReference
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireUpbitOrderReference_UuidCopyWith<WireUpbitOrderReference_Uuid> get copyWith => _$WireUpbitOrderReference_UuidCopyWithImpl<WireUpbitOrderReference_Uuid>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitOrderReference_Uuid&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireUpbitOrderReference.uuid(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireUpbitOrderReference_UuidCopyWith<$Res> implements $WireUpbitOrderReferenceCopyWith<$Res> {
  factory $WireUpbitOrderReference_UuidCopyWith(WireUpbitOrderReference_Uuid value, $Res Function(WireUpbitOrderReference_Uuid) _then) = _$WireUpbitOrderReference_UuidCopyWithImpl;
@override @useResult
$Res call({
 String field0
});




}
/// @nodoc
class _$WireUpbitOrderReference_UuidCopyWithImpl<$Res>
    implements $WireUpbitOrderReference_UuidCopyWith<$Res> {
  _$WireUpbitOrderReference_UuidCopyWithImpl(this._self, this._then);

  final WireUpbitOrderReference_Uuid _self;
  final $Res Function(WireUpbitOrderReference_Uuid) _then;

/// Create a copy of WireUpbitOrderReference
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireUpbitOrderReference_Uuid(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class WireUpbitOrderReference_Identifier extends WireUpbitOrderReference {
  const WireUpbitOrderReference_Identifier(this.field0): super._();


@override final  String field0;

/// Create a copy of WireUpbitOrderReference
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireUpbitOrderReference_IdentifierCopyWith<WireUpbitOrderReference_Identifier> get copyWith => _$WireUpbitOrderReference_IdentifierCopyWithImpl<WireUpbitOrderReference_Identifier>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitOrderReference_Identifier&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireUpbitOrderReference.identifier(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireUpbitOrderReference_IdentifierCopyWith<$Res> implements $WireUpbitOrderReferenceCopyWith<$Res> {
  factory $WireUpbitOrderReference_IdentifierCopyWith(WireUpbitOrderReference_Identifier value, $Res Function(WireUpbitOrderReference_Identifier) _then) = _$WireUpbitOrderReference_IdentifierCopyWithImpl;
@override @useResult
$Res call({
 String field0
});




}
/// @nodoc
class _$WireUpbitOrderReference_IdentifierCopyWithImpl<$Res>
    implements $WireUpbitOrderReference_IdentifierCopyWith<$Res> {
  _$WireUpbitOrderReference_IdentifierCopyWithImpl(this._self, this._then);

  final WireUpbitOrderReference_Identifier _self;
  final $Res Function(WireUpbitOrderReference_Identifier) _then;

/// Create a copy of WireUpbitOrderReference
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireUpbitOrderReference_Identifier(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$WireUpbitOrderVolume {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitOrderVolume);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireUpbitOrderVolume()';
}


}

/// @nodoc
class $WireUpbitOrderVolumeCopyWith<$Res>  {
$WireUpbitOrderVolumeCopyWith(WireUpbitOrderVolume _, $Res Function(WireUpbitOrderVolume) __);
}


/// Adds pattern-matching-related methods to [WireUpbitOrderVolume].
extension WireUpbitOrderVolumePatterns on WireUpbitOrderVolume {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireUpbitOrderVolume_Amount value)?  amount,TResult Function( WireUpbitOrderVolume_RemainOnly value)?  remainOnly,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireUpbitOrderVolume_Amount() when amount != null:
return amount(_that);case WireUpbitOrderVolume_RemainOnly() when remainOnly != null:
return remainOnly(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireUpbitOrderVolume_Amount value)  amount,required TResult Function( WireUpbitOrderVolume_RemainOnly value)  remainOnly,}){
final _that = this;
switch (_that) {
case WireUpbitOrderVolume_Amount():
return amount(_that);case WireUpbitOrderVolume_RemainOnly():
return remainOnly(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireUpbitOrderVolume_Amount value)?  amount,TResult? Function( WireUpbitOrderVolume_RemainOnly value)?  remainOnly,}){
final _that = this;
switch (_that) {
case WireUpbitOrderVolume_Amount() when amount != null:
return amount(_that);case WireUpbitOrderVolume_RemainOnly() when remainOnly != null:
return remainOnly(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String field0)?  amount,TResult Function()?  remainOnly,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireUpbitOrderVolume_Amount() when amount != null:
return amount(_that.field0);case WireUpbitOrderVolume_RemainOnly() when remainOnly != null:
return remainOnly();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String field0)  amount,required TResult Function()  remainOnly,}) {final _that = this;
switch (_that) {
case WireUpbitOrderVolume_Amount():
return amount(_that.field0);case WireUpbitOrderVolume_RemainOnly():
return remainOnly();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String field0)?  amount,TResult? Function()?  remainOnly,}) {final _that = this;
switch (_that) {
case WireUpbitOrderVolume_Amount() when amount != null:
return amount(_that.field0);case WireUpbitOrderVolume_RemainOnly() when remainOnly != null:
return remainOnly();case _:
  return null;

}
}

}

/// @nodoc


class WireUpbitOrderVolume_Amount extends WireUpbitOrderVolume {
  const WireUpbitOrderVolume_Amount(this.field0): super._();


 final  String field0;

/// Create a copy of WireUpbitOrderVolume
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireUpbitOrderVolume_AmountCopyWith<WireUpbitOrderVolume_Amount> get copyWith => _$WireUpbitOrderVolume_AmountCopyWithImpl<WireUpbitOrderVolume_Amount>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitOrderVolume_Amount&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireUpbitOrderVolume.amount(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireUpbitOrderVolume_AmountCopyWith<$Res> implements $WireUpbitOrderVolumeCopyWith<$Res> {
  factory $WireUpbitOrderVolume_AmountCopyWith(WireUpbitOrderVolume_Amount value, $Res Function(WireUpbitOrderVolume_Amount) _then) = _$WireUpbitOrderVolume_AmountCopyWithImpl;
@useResult
$Res call({
 String field0
});




}
/// @nodoc
class _$WireUpbitOrderVolume_AmountCopyWithImpl<$Res>
    implements $WireUpbitOrderVolume_AmountCopyWith<$Res> {
  _$WireUpbitOrderVolume_AmountCopyWithImpl(this._self, this._then);

  final WireUpbitOrderVolume_Amount _self;
  final $Res Function(WireUpbitOrderVolume_Amount) _then;

/// Create a copy of WireUpbitOrderVolume
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireUpbitOrderVolume_Amount(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class WireUpbitOrderVolume_RemainOnly extends WireUpbitOrderVolume {
  const WireUpbitOrderVolume_RemainOnly(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireUpbitOrderVolume_RemainOnly);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireUpbitOrderVolume.remainOnly()';
}


}




/// @nodoc
mixin _$WireWithdrawalFee {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireWithdrawalFee);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireWithdrawalFee()';
}


}

/// @nodoc
class $WireWithdrawalFeeCopyWith<$Res>  {
$WireWithdrawalFeeCopyWith(WireWithdrawalFee _, $Res Function(WireWithdrawalFee) __);
}


/// Adds pattern-matching-related methods to [WireWithdrawalFee].
extension WireWithdrawalFeePatterns on WireWithdrawalFee {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireWithdrawalFee_Fixed value)?  fixed,TResult Function( WireWithdrawalFee_Rate value)?  rate,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireWithdrawalFee_Fixed() when fixed != null:
return fixed(_that);case WireWithdrawalFee_Rate() when rate != null:
return rate(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireWithdrawalFee_Fixed value)  fixed,required TResult Function( WireWithdrawalFee_Rate value)  rate,}){
final _that = this;
switch (_that) {
case WireWithdrawalFee_Fixed():
return fixed(_that);case WireWithdrawalFee_Rate():
return rate(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireWithdrawalFee_Fixed value)?  fixed,TResult? Function( WireWithdrawalFee_Rate value)?  rate,}){
final _that = this;
switch (_that) {
case WireWithdrawalFee_Fixed() when fixed != null:
return fixed(_that);case WireWithdrawalFee_Rate() when rate != null:
return rate(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String field0)?  fixed,TResult Function( String rate,  String? minimum,  String? maximum)?  rate,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireWithdrawalFee_Fixed() when fixed != null:
return fixed(_that.field0);case WireWithdrawalFee_Rate() when rate != null:
return rate(_that.rate,_that.minimum,_that.maximum);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String field0)  fixed,required TResult Function( String rate,  String? minimum,  String? maximum)  rate,}) {final _that = this;
switch (_that) {
case WireWithdrawalFee_Fixed():
return fixed(_that.field0);case WireWithdrawalFee_Rate():
return rate(_that.rate,_that.minimum,_that.maximum);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String field0)?  fixed,TResult? Function( String rate,  String? minimum,  String? maximum)?  rate,}) {final _that = this;
switch (_that) {
case WireWithdrawalFee_Fixed() when fixed != null:
return fixed(_that.field0);case WireWithdrawalFee_Rate() when rate != null:
return rate(_that.rate,_that.minimum,_that.maximum);case _:
  return null;

}
}

}

/// @nodoc


class WireWithdrawalFee_Fixed extends WireWithdrawalFee {
  const WireWithdrawalFee_Fixed(this.field0): super._();


 final  String field0;

/// Create a copy of WireWithdrawalFee
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireWithdrawalFee_FixedCopyWith<WireWithdrawalFee_Fixed> get copyWith => _$WireWithdrawalFee_FixedCopyWithImpl<WireWithdrawalFee_Fixed>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireWithdrawalFee_Fixed&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireWithdrawalFee.fixed(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireWithdrawalFee_FixedCopyWith<$Res> implements $WireWithdrawalFeeCopyWith<$Res> {
  factory $WireWithdrawalFee_FixedCopyWith(WireWithdrawalFee_Fixed value, $Res Function(WireWithdrawalFee_Fixed) _then) = _$WireWithdrawalFee_FixedCopyWithImpl;
@useResult
$Res call({
 String field0
});




}
/// @nodoc
class _$WireWithdrawalFee_FixedCopyWithImpl<$Res>
    implements $WireWithdrawalFee_FixedCopyWith<$Res> {
  _$WireWithdrawalFee_FixedCopyWithImpl(this._self, this._then);

  final WireWithdrawalFee_Fixed _self;
  final $Res Function(WireWithdrawalFee_Fixed) _then;

/// Create a copy of WireWithdrawalFee
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireWithdrawalFee_Fixed(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class WireWithdrawalFee_Rate extends WireWithdrawalFee {
  const WireWithdrawalFee_Rate({required this.rate, this.minimum, this.maximum}): super._();


 final  String rate;
 final  String? minimum;
 final  String? maximum;

/// Create a copy of WireWithdrawalFee
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireWithdrawalFee_RateCopyWith<WireWithdrawalFee_Rate> get copyWith => _$WireWithdrawalFee_RateCopyWithImpl<WireWithdrawalFee_Rate>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireWithdrawalFee_Rate&&(identical(other.rate, rate) || other.rate == rate)&&(identical(other.minimum, minimum) || other.minimum == minimum)&&(identical(other.maximum, maximum) || other.maximum == maximum));
}


@override
int get hashCode => Object.hash(runtimeType,rate,minimum,maximum);

@override
String toString() {
  return 'WireWithdrawalFee.rate(rate: $rate, minimum: $minimum, maximum: $maximum)';
}


}

/// @nodoc
abstract mixin class $WireWithdrawalFee_RateCopyWith<$Res> implements $WireWithdrawalFeeCopyWith<$Res> {
  factory $WireWithdrawalFee_RateCopyWith(WireWithdrawalFee_Rate value, $Res Function(WireWithdrawalFee_Rate) _then) = _$WireWithdrawalFee_RateCopyWithImpl;
@useResult
$Res call({
 String rate, String? minimum, String? maximum
});




}
/// @nodoc
class _$WireWithdrawalFee_RateCopyWithImpl<$Res>
    implements $WireWithdrawalFee_RateCopyWith<$Res> {
  _$WireWithdrawalFee_RateCopyWithImpl(this._self, this._then);

  final WireWithdrawalFee_Rate _self;
  final $Res Function(WireWithdrawalFee_Rate) _then;

/// Create a copy of WireWithdrawalFee
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? rate = null,Object? minimum = freezed,Object? maximum = freezed,}) {
  return _then(WireWithdrawalFee_Rate(
rate: null == rate ? _self.rate : rate // ignore: cast_nullable_to_non_nullable
as String,minimum: freezed == minimum ? _self.minimum : minimum // ignore: cast_nullable_to_non_nullable
as String?,maximum: freezed == maximum ? _self.maximum : maximum // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

// dart format on

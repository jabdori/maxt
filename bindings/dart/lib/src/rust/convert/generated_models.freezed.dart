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

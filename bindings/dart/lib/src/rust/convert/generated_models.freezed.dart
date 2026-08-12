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

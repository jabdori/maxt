// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'adapter.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AdapterCall {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AdapterCall()';
}


}

/// @nodoc
class $AdapterCallCopyWith<$Res>  {
$AdapterCallCopyWith(AdapterCall _, $Res Function(AdapterCall) __);
}


/// Adds pattern-matching-related methods to [AdapterCall].
extension AdapterCallPatterns on AdapterCall {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( AdapterCall_Markets value)?  markets,TResult Function( AdapterCall_Trades value)?  trades,TResult Function( AdapterCall_OrderBook value)?  orderBook,TResult Function( AdapterCall_Ticker value)?  ticker,TResult Function( AdapterCall_Candles value)?  candles,TResult Function( AdapterCall_Balances value)?  balances,TResult Function( AdapterCall_OrderRules value)?  orderRules,TResult Function( AdapterCall_AssetNetworks value)?  assetNetworks,TResult Function( AdapterCall_DepositAddress value)?  depositAddress,TResult Function( AdapterCall_CreateDepositAddress value)?  createDepositAddress,TResult Function( AdapterCall_PrepareWithdrawal value)?  prepareWithdrawal,TResult Function( AdapterCall_Withdraw value)?  withdraw,TResult Function( AdapterCall_Deposits value)?  deposits,TResult Function( AdapterCall_Withdrawals value)?  withdrawals,TResult Function( AdapterCall_OpenOrders value)?  openOrders,TResult Function( AdapterCall_Order value)?  order,TResult Function( AdapterCall_OrderByClientId value)?  orderByClientId,TResult Function( AdapterCall_OrdersByIds value)?  ordersByIds,TResult Function( AdapterCall_OrderHistory value)?  orderHistory,TResult Function( AdapterCall_PlaceOrder value)?  placeOrder,TResult Function( AdapterCall_CancelOrder value)?  cancelOrder,TResult Function( AdapterCall_CancelOrderByClientId value)?  cancelOrderByClientId,TResult Function( AdapterCall_CancelOrders value)?  cancelOrders,TResult Function( AdapterCall_Positions value)?  positions,TResult Function( AdapterCall_MarginSummary value)?  marginSummary,TResult Function( AdapterCall_FundingRates value)?  fundingRates,TResult Function( AdapterCall_FundingPayments value)?  fundingPayments,TResult Function( AdapterCall_SetMargin value)?  setMargin,TResult Function( AdapterCall_Subscribe value)?  subscribe,TResult Function( AdapterCall_SubscribeAccount value)?  subscribeAccount,TResult Function( AdapterCall_CancelStream value)?  cancelStream,required TResult orElse(),}){
final _that = this;
switch (_that) {
case AdapterCall_Markets() when markets != null:
return markets(_that);case AdapterCall_Trades() when trades != null:
return trades(_that);case AdapterCall_OrderBook() when orderBook != null:
return orderBook(_that);case AdapterCall_Ticker() when ticker != null:
return ticker(_that);case AdapterCall_Candles() when candles != null:
return candles(_that);case AdapterCall_Balances() when balances != null:
return balances(_that);case AdapterCall_OrderRules() when orderRules != null:
return orderRules(_that);case AdapterCall_AssetNetworks() when assetNetworks != null:
return assetNetworks(_that);case AdapterCall_DepositAddress() when depositAddress != null:
return depositAddress(_that);case AdapterCall_CreateDepositAddress() when createDepositAddress != null:
return createDepositAddress(_that);case AdapterCall_PrepareWithdrawal() when prepareWithdrawal != null:
return prepareWithdrawal(_that);case AdapterCall_Withdraw() when withdraw != null:
return withdraw(_that);case AdapterCall_Deposits() when deposits != null:
return deposits(_that);case AdapterCall_Withdrawals() when withdrawals != null:
return withdrawals(_that);case AdapterCall_OpenOrders() when openOrders != null:
return openOrders(_that);case AdapterCall_Order() when order != null:
return order(_that);case AdapterCall_OrderByClientId() when orderByClientId != null:
return orderByClientId(_that);case AdapterCall_OrdersByIds() when ordersByIds != null:
return ordersByIds(_that);case AdapterCall_OrderHistory() when orderHistory != null:
return orderHistory(_that);case AdapterCall_PlaceOrder() when placeOrder != null:
return placeOrder(_that);case AdapterCall_CancelOrder() when cancelOrder != null:
return cancelOrder(_that);case AdapterCall_CancelOrderByClientId() when cancelOrderByClientId != null:
return cancelOrderByClientId(_that);case AdapterCall_CancelOrders() when cancelOrders != null:
return cancelOrders(_that);case AdapterCall_Positions() when positions != null:
return positions(_that);case AdapterCall_MarginSummary() when marginSummary != null:
return marginSummary(_that);case AdapterCall_FundingRates() when fundingRates != null:
return fundingRates(_that);case AdapterCall_FundingPayments() when fundingPayments != null:
return fundingPayments(_that);case AdapterCall_SetMargin() when setMargin != null:
return setMargin(_that);case AdapterCall_Subscribe() when subscribe != null:
return subscribe(_that);case AdapterCall_SubscribeAccount() when subscribeAccount != null:
return subscribeAccount(_that);case AdapterCall_CancelStream() when cancelStream != null:
return cancelStream(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( AdapterCall_Markets value)  markets,required TResult Function( AdapterCall_Trades value)  trades,required TResult Function( AdapterCall_OrderBook value)  orderBook,required TResult Function( AdapterCall_Ticker value)  ticker,required TResult Function( AdapterCall_Candles value)  candles,required TResult Function( AdapterCall_Balances value)  balances,required TResult Function( AdapterCall_OrderRules value)  orderRules,required TResult Function( AdapterCall_AssetNetworks value)  assetNetworks,required TResult Function( AdapterCall_DepositAddress value)  depositAddress,required TResult Function( AdapterCall_CreateDepositAddress value)  createDepositAddress,required TResult Function( AdapterCall_PrepareWithdrawal value)  prepareWithdrawal,required TResult Function( AdapterCall_Withdraw value)  withdraw,required TResult Function( AdapterCall_Deposits value)  deposits,required TResult Function( AdapterCall_Withdrawals value)  withdrawals,required TResult Function( AdapterCall_OpenOrders value)  openOrders,required TResult Function( AdapterCall_Order value)  order,required TResult Function( AdapterCall_OrderByClientId value)  orderByClientId,required TResult Function( AdapterCall_OrdersByIds value)  ordersByIds,required TResult Function( AdapterCall_OrderHistory value)  orderHistory,required TResult Function( AdapterCall_PlaceOrder value)  placeOrder,required TResult Function( AdapterCall_CancelOrder value)  cancelOrder,required TResult Function( AdapterCall_CancelOrderByClientId value)  cancelOrderByClientId,required TResult Function( AdapterCall_CancelOrders value)  cancelOrders,required TResult Function( AdapterCall_Positions value)  positions,required TResult Function( AdapterCall_MarginSummary value)  marginSummary,required TResult Function( AdapterCall_FundingRates value)  fundingRates,required TResult Function( AdapterCall_FundingPayments value)  fundingPayments,required TResult Function( AdapterCall_SetMargin value)  setMargin,required TResult Function( AdapterCall_Subscribe value)  subscribe,required TResult Function( AdapterCall_SubscribeAccount value)  subscribeAccount,required TResult Function( AdapterCall_CancelStream value)  cancelStream,}){
final _that = this;
switch (_that) {
case AdapterCall_Markets():
return markets(_that);case AdapterCall_Trades():
return trades(_that);case AdapterCall_OrderBook():
return orderBook(_that);case AdapterCall_Ticker():
return ticker(_that);case AdapterCall_Candles():
return candles(_that);case AdapterCall_Balances():
return balances(_that);case AdapterCall_OrderRules():
return orderRules(_that);case AdapterCall_AssetNetworks():
return assetNetworks(_that);case AdapterCall_DepositAddress():
return depositAddress(_that);case AdapterCall_CreateDepositAddress():
return createDepositAddress(_that);case AdapterCall_PrepareWithdrawal():
return prepareWithdrawal(_that);case AdapterCall_Withdraw():
return withdraw(_that);case AdapterCall_Deposits():
return deposits(_that);case AdapterCall_Withdrawals():
return withdrawals(_that);case AdapterCall_OpenOrders():
return openOrders(_that);case AdapterCall_Order():
return order(_that);case AdapterCall_OrderByClientId():
return orderByClientId(_that);case AdapterCall_OrdersByIds():
return ordersByIds(_that);case AdapterCall_OrderHistory():
return orderHistory(_that);case AdapterCall_PlaceOrder():
return placeOrder(_that);case AdapterCall_CancelOrder():
return cancelOrder(_that);case AdapterCall_CancelOrderByClientId():
return cancelOrderByClientId(_that);case AdapterCall_CancelOrders():
return cancelOrders(_that);case AdapterCall_Positions():
return positions(_that);case AdapterCall_MarginSummary():
return marginSummary(_that);case AdapterCall_FundingRates():
return fundingRates(_that);case AdapterCall_FundingPayments():
return fundingPayments(_that);case AdapterCall_SetMargin():
return setMargin(_that);case AdapterCall_Subscribe():
return subscribe(_that);case AdapterCall_SubscribeAccount():
return subscribeAccount(_that);case AdapterCall_CancelStream():
return cancelStream(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( AdapterCall_Markets value)?  markets,TResult? Function( AdapterCall_Trades value)?  trades,TResult? Function( AdapterCall_OrderBook value)?  orderBook,TResult? Function( AdapterCall_Ticker value)?  ticker,TResult? Function( AdapterCall_Candles value)?  candles,TResult? Function( AdapterCall_Balances value)?  balances,TResult? Function( AdapterCall_OrderRules value)?  orderRules,TResult? Function( AdapterCall_AssetNetworks value)?  assetNetworks,TResult? Function( AdapterCall_DepositAddress value)?  depositAddress,TResult? Function( AdapterCall_CreateDepositAddress value)?  createDepositAddress,TResult? Function( AdapterCall_PrepareWithdrawal value)?  prepareWithdrawal,TResult? Function( AdapterCall_Withdraw value)?  withdraw,TResult? Function( AdapterCall_Deposits value)?  deposits,TResult? Function( AdapterCall_Withdrawals value)?  withdrawals,TResult? Function( AdapterCall_OpenOrders value)?  openOrders,TResult? Function( AdapterCall_Order value)?  order,TResult? Function( AdapterCall_OrderByClientId value)?  orderByClientId,TResult? Function( AdapterCall_OrdersByIds value)?  ordersByIds,TResult? Function( AdapterCall_OrderHistory value)?  orderHistory,TResult? Function( AdapterCall_PlaceOrder value)?  placeOrder,TResult? Function( AdapterCall_CancelOrder value)?  cancelOrder,TResult? Function( AdapterCall_CancelOrderByClientId value)?  cancelOrderByClientId,TResult? Function( AdapterCall_CancelOrders value)?  cancelOrders,TResult? Function( AdapterCall_Positions value)?  positions,TResult? Function( AdapterCall_MarginSummary value)?  marginSummary,TResult? Function( AdapterCall_FundingRates value)?  fundingRates,TResult? Function( AdapterCall_FundingPayments value)?  fundingPayments,TResult? Function( AdapterCall_SetMargin value)?  setMargin,TResult? Function( AdapterCall_Subscribe value)?  subscribe,TResult? Function( AdapterCall_SubscribeAccount value)?  subscribeAccount,TResult? Function( AdapterCall_CancelStream value)?  cancelStream,}){
final _that = this;
switch (_that) {
case AdapterCall_Markets() when markets != null:
return markets(_that);case AdapterCall_Trades() when trades != null:
return trades(_that);case AdapterCall_OrderBook() when orderBook != null:
return orderBook(_that);case AdapterCall_Ticker() when ticker != null:
return ticker(_that);case AdapterCall_Candles() when candles != null:
return candles(_that);case AdapterCall_Balances() when balances != null:
return balances(_that);case AdapterCall_OrderRules() when orderRules != null:
return orderRules(_that);case AdapterCall_AssetNetworks() when assetNetworks != null:
return assetNetworks(_that);case AdapterCall_DepositAddress() when depositAddress != null:
return depositAddress(_that);case AdapterCall_CreateDepositAddress() when createDepositAddress != null:
return createDepositAddress(_that);case AdapterCall_PrepareWithdrawal() when prepareWithdrawal != null:
return prepareWithdrawal(_that);case AdapterCall_Withdraw() when withdraw != null:
return withdraw(_that);case AdapterCall_Deposits() when deposits != null:
return deposits(_that);case AdapterCall_Withdrawals() when withdrawals != null:
return withdrawals(_that);case AdapterCall_OpenOrders() when openOrders != null:
return openOrders(_that);case AdapterCall_Order() when order != null:
return order(_that);case AdapterCall_OrderByClientId() when orderByClientId != null:
return orderByClientId(_that);case AdapterCall_OrdersByIds() when ordersByIds != null:
return ordersByIds(_that);case AdapterCall_OrderHistory() when orderHistory != null:
return orderHistory(_that);case AdapterCall_PlaceOrder() when placeOrder != null:
return placeOrder(_that);case AdapterCall_CancelOrder() when cancelOrder != null:
return cancelOrder(_that);case AdapterCall_CancelOrderByClientId() when cancelOrderByClientId != null:
return cancelOrderByClientId(_that);case AdapterCall_CancelOrders() when cancelOrders != null:
return cancelOrders(_that);case AdapterCall_Positions() when positions != null:
return positions(_that);case AdapterCall_MarginSummary() when marginSummary != null:
return marginSummary(_that);case AdapterCall_FundingRates() when fundingRates != null:
return fundingRates(_that);case AdapterCall_FundingPayments() when fundingPayments != null:
return fundingPayments(_that);case AdapterCall_SetMargin() when setMargin != null:
return setMargin(_that);case AdapterCall_Subscribe() when subscribe != null:
return subscribe(_that);case AdapterCall_SubscribeAccount() when subscribeAccount != null:
return subscribeAccount(_that);case AdapterCall_CancelStream() when cancelStream != null:
return cancelStream(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( WireMarketKind kind)?  markets,TResult Function( WireMarket market,  int? limit)?  trades,TResult Function( WireMarket market,  int? depth)?  orderBook,TResult Function( WireMarket market)?  ticker,TResult Function( WireCandleRequest request)?  candles,TResult Function()?  balances,TResult Function( WireMarket market)?  orderRules,TResult Function( String asset)?  assetNetworks,TResult Function( WireDepositAddressRequest request)?  depositAddress,TResult Function( WireDepositAddressRequest request)?  createDepositAddress,TResult Function( WireWithdrawRequest request)?  prepareWithdrawal,TResult Function( WireWithdrawRequest request)?  withdraw,TResult Function( WireTransferHistoryRequest request)?  deposits,TResult Function( WireTransferHistoryRequest request)?  withdrawals,TResult Function( WireMarket? market)?  openOrders,TResult Function( WireMarket market,  String orderId)?  order,TResult Function( WireMarket market,  String clientId)?  orderByClientId,TResult Function( WireOrderLookupRequest request)?  ordersByIds,TResult Function( WireOrderHistoryRequest request)?  orderHistory,TResult Function( WireOrderRequest request)?  placeOrder,TResult Function( WireMarket market,  String orderId)?  cancelOrder,TResult Function( WireMarket market,  String clientId)?  cancelOrderByClientId,TResult Function( WireCancelOrdersRequest request)?  cancelOrders,TResult Function( WireMarket? market)?  positions,TResult Function()?  marginSummary,TResult Function( WireHistoryRequest request)?  fundingRates,TResult Function( WireHistoryRequest request)?  fundingPayments,TResult Function( WireMarginRequest request)?  setMargin,TResult Function( String streamId,  WireSubscription subscription,  WireStreamConfig config,  MarketStreamSink sink)?  subscribe,TResult Function( String streamId,  WireStreamConfig config,  AccountStreamSink sink)?  subscribeAccount,TResult Function( String streamId)?  cancelStream,required TResult orElse(),}) {final _that = this;
switch (_that) {
case AdapterCall_Markets() when markets != null:
return markets(_that.kind);case AdapterCall_Trades() when trades != null:
return trades(_that.market,_that.limit);case AdapterCall_OrderBook() when orderBook != null:
return orderBook(_that.market,_that.depth);case AdapterCall_Ticker() when ticker != null:
return ticker(_that.market);case AdapterCall_Candles() when candles != null:
return candles(_that.request);case AdapterCall_Balances() when balances != null:
return balances();case AdapterCall_OrderRules() when orderRules != null:
return orderRules(_that.market);case AdapterCall_AssetNetworks() when assetNetworks != null:
return assetNetworks(_that.asset);case AdapterCall_DepositAddress() when depositAddress != null:
return depositAddress(_that.request);case AdapterCall_CreateDepositAddress() when createDepositAddress != null:
return createDepositAddress(_that.request);case AdapterCall_PrepareWithdrawal() when prepareWithdrawal != null:
return prepareWithdrawal(_that.request);case AdapterCall_Withdraw() when withdraw != null:
return withdraw(_that.request);case AdapterCall_Deposits() when deposits != null:
return deposits(_that.request);case AdapterCall_Withdrawals() when withdrawals != null:
return withdrawals(_that.request);case AdapterCall_OpenOrders() when openOrders != null:
return openOrders(_that.market);case AdapterCall_Order() when order != null:
return order(_that.market,_that.orderId);case AdapterCall_OrderByClientId() when orderByClientId != null:
return orderByClientId(_that.market,_that.clientId);case AdapterCall_OrdersByIds() when ordersByIds != null:
return ordersByIds(_that.request);case AdapterCall_OrderHistory() when orderHistory != null:
return orderHistory(_that.request);case AdapterCall_PlaceOrder() when placeOrder != null:
return placeOrder(_that.request);case AdapterCall_CancelOrder() when cancelOrder != null:
return cancelOrder(_that.market,_that.orderId);case AdapterCall_CancelOrderByClientId() when cancelOrderByClientId != null:
return cancelOrderByClientId(_that.market,_that.clientId);case AdapterCall_CancelOrders() when cancelOrders != null:
return cancelOrders(_that.request);case AdapterCall_Positions() when positions != null:
return positions(_that.market);case AdapterCall_MarginSummary() when marginSummary != null:
return marginSummary();case AdapterCall_FundingRates() when fundingRates != null:
return fundingRates(_that.request);case AdapterCall_FundingPayments() when fundingPayments != null:
return fundingPayments(_that.request);case AdapterCall_SetMargin() when setMargin != null:
return setMargin(_that.request);case AdapterCall_Subscribe() when subscribe != null:
return subscribe(_that.streamId,_that.subscription,_that.config,_that.sink);case AdapterCall_SubscribeAccount() when subscribeAccount != null:
return subscribeAccount(_that.streamId,_that.config,_that.sink);case AdapterCall_CancelStream() when cancelStream != null:
return cancelStream(_that.streamId);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( WireMarketKind kind)  markets,required TResult Function( WireMarket market,  int? limit)  trades,required TResult Function( WireMarket market,  int? depth)  orderBook,required TResult Function( WireMarket market)  ticker,required TResult Function( WireCandleRequest request)  candles,required TResult Function()  balances,required TResult Function( WireMarket market)  orderRules,required TResult Function( String asset)  assetNetworks,required TResult Function( WireDepositAddressRequest request)  depositAddress,required TResult Function( WireDepositAddressRequest request)  createDepositAddress,required TResult Function( WireWithdrawRequest request)  prepareWithdrawal,required TResult Function( WireWithdrawRequest request)  withdraw,required TResult Function( WireTransferHistoryRequest request)  deposits,required TResult Function( WireTransferHistoryRequest request)  withdrawals,required TResult Function( WireMarket? market)  openOrders,required TResult Function( WireMarket market,  String orderId)  order,required TResult Function( WireMarket market,  String clientId)  orderByClientId,required TResult Function( WireOrderLookupRequest request)  ordersByIds,required TResult Function( WireOrderHistoryRequest request)  orderHistory,required TResult Function( WireOrderRequest request)  placeOrder,required TResult Function( WireMarket market,  String orderId)  cancelOrder,required TResult Function( WireMarket market,  String clientId)  cancelOrderByClientId,required TResult Function( WireCancelOrdersRequest request)  cancelOrders,required TResult Function( WireMarket? market)  positions,required TResult Function()  marginSummary,required TResult Function( WireHistoryRequest request)  fundingRates,required TResult Function( WireHistoryRequest request)  fundingPayments,required TResult Function( WireMarginRequest request)  setMargin,required TResult Function( String streamId,  WireSubscription subscription,  WireStreamConfig config,  MarketStreamSink sink)  subscribe,required TResult Function( String streamId,  WireStreamConfig config,  AccountStreamSink sink)  subscribeAccount,required TResult Function( String streamId)  cancelStream,}) {final _that = this;
switch (_that) {
case AdapterCall_Markets():
return markets(_that.kind);case AdapterCall_Trades():
return trades(_that.market,_that.limit);case AdapterCall_OrderBook():
return orderBook(_that.market,_that.depth);case AdapterCall_Ticker():
return ticker(_that.market);case AdapterCall_Candles():
return candles(_that.request);case AdapterCall_Balances():
return balances();case AdapterCall_OrderRules():
return orderRules(_that.market);case AdapterCall_AssetNetworks():
return assetNetworks(_that.asset);case AdapterCall_DepositAddress():
return depositAddress(_that.request);case AdapterCall_CreateDepositAddress():
return createDepositAddress(_that.request);case AdapterCall_PrepareWithdrawal():
return prepareWithdrawal(_that.request);case AdapterCall_Withdraw():
return withdraw(_that.request);case AdapterCall_Deposits():
return deposits(_that.request);case AdapterCall_Withdrawals():
return withdrawals(_that.request);case AdapterCall_OpenOrders():
return openOrders(_that.market);case AdapterCall_Order():
return order(_that.market,_that.orderId);case AdapterCall_OrderByClientId():
return orderByClientId(_that.market,_that.clientId);case AdapterCall_OrdersByIds():
return ordersByIds(_that.request);case AdapterCall_OrderHistory():
return orderHistory(_that.request);case AdapterCall_PlaceOrder():
return placeOrder(_that.request);case AdapterCall_CancelOrder():
return cancelOrder(_that.market,_that.orderId);case AdapterCall_CancelOrderByClientId():
return cancelOrderByClientId(_that.market,_that.clientId);case AdapterCall_CancelOrders():
return cancelOrders(_that.request);case AdapterCall_Positions():
return positions(_that.market);case AdapterCall_MarginSummary():
return marginSummary();case AdapterCall_FundingRates():
return fundingRates(_that.request);case AdapterCall_FundingPayments():
return fundingPayments(_that.request);case AdapterCall_SetMargin():
return setMargin(_that.request);case AdapterCall_Subscribe():
return subscribe(_that.streamId,_that.subscription,_that.config,_that.sink);case AdapterCall_SubscribeAccount():
return subscribeAccount(_that.streamId,_that.config,_that.sink);case AdapterCall_CancelStream():
return cancelStream(_that.streamId);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( WireMarketKind kind)?  markets,TResult? Function( WireMarket market,  int? limit)?  trades,TResult? Function( WireMarket market,  int? depth)?  orderBook,TResult? Function( WireMarket market)?  ticker,TResult? Function( WireCandleRequest request)?  candles,TResult? Function()?  balances,TResult? Function( WireMarket market)?  orderRules,TResult? Function( String asset)?  assetNetworks,TResult? Function( WireDepositAddressRequest request)?  depositAddress,TResult? Function( WireDepositAddressRequest request)?  createDepositAddress,TResult? Function( WireWithdrawRequest request)?  prepareWithdrawal,TResult? Function( WireWithdrawRequest request)?  withdraw,TResult? Function( WireTransferHistoryRequest request)?  deposits,TResult? Function( WireTransferHistoryRequest request)?  withdrawals,TResult? Function( WireMarket? market)?  openOrders,TResult? Function( WireMarket market,  String orderId)?  order,TResult? Function( WireMarket market,  String clientId)?  orderByClientId,TResult? Function( WireOrderLookupRequest request)?  ordersByIds,TResult? Function( WireOrderHistoryRequest request)?  orderHistory,TResult? Function( WireOrderRequest request)?  placeOrder,TResult? Function( WireMarket market,  String orderId)?  cancelOrder,TResult? Function( WireMarket market,  String clientId)?  cancelOrderByClientId,TResult? Function( WireCancelOrdersRequest request)?  cancelOrders,TResult? Function( WireMarket? market)?  positions,TResult? Function()?  marginSummary,TResult? Function( WireHistoryRequest request)?  fundingRates,TResult? Function( WireHistoryRequest request)?  fundingPayments,TResult? Function( WireMarginRequest request)?  setMargin,TResult? Function( String streamId,  WireSubscription subscription,  WireStreamConfig config,  MarketStreamSink sink)?  subscribe,TResult? Function( String streamId,  WireStreamConfig config,  AccountStreamSink sink)?  subscribeAccount,TResult? Function( String streamId)?  cancelStream,}) {final _that = this;
switch (_that) {
case AdapterCall_Markets() when markets != null:
return markets(_that.kind);case AdapterCall_Trades() when trades != null:
return trades(_that.market,_that.limit);case AdapterCall_OrderBook() when orderBook != null:
return orderBook(_that.market,_that.depth);case AdapterCall_Ticker() when ticker != null:
return ticker(_that.market);case AdapterCall_Candles() when candles != null:
return candles(_that.request);case AdapterCall_Balances() when balances != null:
return balances();case AdapterCall_OrderRules() when orderRules != null:
return orderRules(_that.market);case AdapterCall_AssetNetworks() when assetNetworks != null:
return assetNetworks(_that.asset);case AdapterCall_DepositAddress() when depositAddress != null:
return depositAddress(_that.request);case AdapterCall_CreateDepositAddress() when createDepositAddress != null:
return createDepositAddress(_that.request);case AdapterCall_PrepareWithdrawal() when prepareWithdrawal != null:
return prepareWithdrawal(_that.request);case AdapterCall_Withdraw() when withdraw != null:
return withdraw(_that.request);case AdapterCall_Deposits() when deposits != null:
return deposits(_that.request);case AdapterCall_Withdrawals() when withdrawals != null:
return withdrawals(_that.request);case AdapterCall_OpenOrders() when openOrders != null:
return openOrders(_that.market);case AdapterCall_Order() when order != null:
return order(_that.market,_that.orderId);case AdapterCall_OrderByClientId() when orderByClientId != null:
return orderByClientId(_that.market,_that.clientId);case AdapterCall_OrdersByIds() when ordersByIds != null:
return ordersByIds(_that.request);case AdapterCall_OrderHistory() when orderHistory != null:
return orderHistory(_that.request);case AdapterCall_PlaceOrder() when placeOrder != null:
return placeOrder(_that.request);case AdapterCall_CancelOrder() when cancelOrder != null:
return cancelOrder(_that.market,_that.orderId);case AdapterCall_CancelOrderByClientId() when cancelOrderByClientId != null:
return cancelOrderByClientId(_that.market,_that.clientId);case AdapterCall_CancelOrders() when cancelOrders != null:
return cancelOrders(_that.request);case AdapterCall_Positions() when positions != null:
return positions(_that.market);case AdapterCall_MarginSummary() when marginSummary != null:
return marginSummary();case AdapterCall_FundingRates() when fundingRates != null:
return fundingRates(_that.request);case AdapterCall_FundingPayments() when fundingPayments != null:
return fundingPayments(_that.request);case AdapterCall_SetMargin() when setMargin != null:
return setMargin(_that.request);case AdapterCall_Subscribe() when subscribe != null:
return subscribe(_that.streamId,_that.subscription,_that.config,_that.sink);case AdapterCall_SubscribeAccount() when subscribeAccount != null:
return subscribeAccount(_that.streamId,_that.config,_that.sink);case AdapterCall_CancelStream() when cancelStream != null:
return cancelStream(_that.streamId);case _:
  return null;

}
}

}

/// @nodoc


class AdapterCall_Markets extends AdapterCall {
  const AdapterCall_Markets({required this.kind}): super._();


 final  WireMarketKind kind;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterCall_MarketsCopyWith<AdapterCall_Markets> get copyWith => _$AdapterCall_MarketsCopyWithImpl<AdapterCall_Markets>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall_Markets&&(identical(other.kind, kind) || other.kind == kind));
}


@override
int get hashCode => Object.hash(runtimeType,kind);

@override
String toString() {
  return 'AdapterCall.markets(kind: $kind)';
}


}

/// @nodoc
abstract mixin class $AdapterCall_MarketsCopyWith<$Res> implements $AdapterCallCopyWith<$Res> {
  factory $AdapterCall_MarketsCopyWith(AdapterCall_Markets value, $Res Function(AdapterCall_Markets) _then) = _$AdapterCall_MarketsCopyWithImpl;
@useResult
$Res call({
 WireMarketKind kind
});




}
/// @nodoc
class _$AdapterCall_MarketsCopyWithImpl<$Res>
    implements $AdapterCall_MarketsCopyWith<$Res> {
  _$AdapterCall_MarketsCopyWithImpl(this._self, this._then);

  final AdapterCall_Markets _self;
  final $Res Function(AdapterCall_Markets) _then;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? kind = null,}) {
  return _then(AdapterCall_Markets(
kind: null == kind ? _self.kind : kind // ignore: cast_nullable_to_non_nullable
as WireMarketKind,
  ));
}


}

/// @nodoc


class AdapterCall_Trades extends AdapterCall {
  const AdapterCall_Trades({required this.market, this.limit}): super._();


 final  WireMarket market;
 final  int? limit;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterCall_TradesCopyWith<AdapterCall_Trades> get copyWith => _$AdapterCall_TradesCopyWithImpl<AdapterCall_Trades>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall_Trades&&(identical(other.market, market) || other.market == market)&&(identical(other.limit, limit) || other.limit == limit));
}


@override
int get hashCode => Object.hash(runtimeType,market,limit);

@override
String toString() {
  return 'AdapterCall.trades(market: $market, limit: $limit)';
}


}

/// @nodoc
abstract mixin class $AdapterCall_TradesCopyWith<$Res> implements $AdapterCallCopyWith<$Res> {
  factory $AdapterCall_TradesCopyWith(AdapterCall_Trades value, $Res Function(AdapterCall_Trades) _then) = _$AdapterCall_TradesCopyWithImpl;
@useResult
$Res call({
 WireMarket market, int? limit
});




}
/// @nodoc
class _$AdapterCall_TradesCopyWithImpl<$Res>
    implements $AdapterCall_TradesCopyWith<$Res> {
  _$AdapterCall_TradesCopyWithImpl(this._self, this._then);

  final AdapterCall_Trades _self;
  final $Res Function(AdapterCall_Trades) _then;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? market = null,Object? limit = freezed,}) {
  return _then(AdapterCall_Trades(
market: null == market ? _self.market : market // ignore: cast_nullable_to_non_nullable
as WireMarket,limit: freezed == limit ? _self.limit : limit // ignore: cast_nullable_to_non_nullable
as int?,
  ));
}


}

/// @nodoc


class AdapterCall_OrderBook extends AdapterCall {
  const AdapterCall_OrderBook({required this.market, this.depth}): super._();


 final  WireMarket market;
 final  int? depth;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterCall_OrderBookCopyWith<AdapterCall_OrderBook> get copyWith => _$AdapterCall_OrderBookCopyWithImpl<AdapterCall_OrderBook>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall_OrderBook&&(identical(other.market, market) || other.market == market)&&(identical(other.depth, depth) || other.depth == depth));
}


@override
int get hashCode => Object.hash(runtimeType,market,depth);

@override
String toString() {
  return 'AdapterCall.orderBook(market: $market, depth: $depth)';
}


}

/// @nodoc
abstract mixin class $AdapterCall_OrderBookCopyWith<$Res> implements $AdapterCallCopyWith<$Res> {
  factory $AdapterCall_OrderBookCopyWith(AdapterCall_OrderBook value, $Res Function(AdapterCall_OrderBook) _then) = _$AdapterCall_OrderBookCopyWithImpl;
@useResult
$Res call({
 WireMarket market, int? depth
});




}
/// @nodoc
class _$AdapterCall_OrderBookCopyWithImpl<$Res>
    implements $AdapterCall_OrderBookCopyWith<$Res> {
  _$AdapterCall_OrderBookCopyWithImpl(this._self, this._then);

  final AdapterCall_OrderBook _self;
  final $Res Function(AdapterCall_OrderBook) _then;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? market = null,Object? depth = freezed,}) {
  return _then(AdapterCall_OrderBook(
market: null == market ? _self.market : market // ignore: cast_nullable_to_non_nullable
as WireMarket,depth: freezed == depth ? _self.depth : depth // ignore: cast_nullable_to_non_nullable
as int?,
  ));
}


}

/// @nodoc


class AdapterCall_Ticker extends AdapterCall {
  const AdapterCall_Ticker({required this.market}): super._();


 final  WireMarket market;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterCall_TickerCopyWith<AdapterCall_Ticker> get copyWith => _$AdapterCall_TickerCopyWithImpl<AdapterCall_Ticker>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall_Ticker&&(identical(other.market, market) || other.market == market));
}


@override
int get hashCode => Object.hash(runtimeType,market);

@override
String toString() {
  return 'AdapterCall.ticker(market: $market)';
}


}

/// @nodoc
abstract mixin class $AdapterCall_TickerCopyWith<$Res> implements $AdapterCallCopyWith<$Res> {
  factory $AdapterCall_TickerCopyWith(AdapterCall_Ticker value, $Res Function(AdapterCall_Ticker) _then) = _$AdapterCall_TickerCopyWithImpl;
@useResult
$Res call({
 WireMarket market
});




}
/// @nodoc
class _$AdapterCall_TickerCopyWithImpl<$Res>
    implements $AdapterCall_TickerCopyWith<$Res> {
  _$AdapterCall_TickerCopyWithImpl(this._self, this._then);

  final AdapterCall_Ticker _self;
  final $Res Function(AdapterCall_Ticker) _then;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? market = null,}) {
  return _then(AdapterCall_Ticker(
market: null == market ? _self.market : market // ignore: cast_nullable_to_non_nullable
as WireMarket,
  ));
}


}

/// @nodoc


class AdapterCall_Candles extends AdapterCall {
  const AdapterCall_Candles({required this.request}): super._();


 final  WireCandleRequest request;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterCall_CandlesCopyWith<AdapterCall_Candles> get copyWith => _$AdapterCall_CandlesCopyWithImpl<AdapterCall_Candles>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall_Candles&&(identical(other.request, request) || other.request == request));
}


@override
int get hashCode => Object.hash(runtimeType,request);

@override
String toString() {
  return 'AdapterCall.candles(request: $request)';
}


}

/// @nodoc
abstract mixin class $AdapterCall_CandlesCopyWith<$Res> implements $AdapterCallCopyWith<$Res> {
  factory $AdapterCall_CandlesCopyWith(AdapterCall_Candles value, $Res Function(AdapterCall_Candles) _then) = _$AdapterCall_CandlesCopyWithImpl;
@useResult
$Res call({
 WireCandleRequest request
});




}
/// @nodoc
class _$AdapterCall_CandlesCopyWithImpl<$Res>
    implements $AdapterCall_CandlesCopyWith<$Res> {
  _$AdapterCall_CandlesCopyWithImpl(this._self, this._then);

  final AdapterCall_Candles _self;
  final $Res Function(AdapterCall_Candles) _then;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? request = null,}) {
  return _then(AdapterCall_Candles(
request: null == request ? _self.request : request // ignore: cast_nullable_to_non_nullable
as WireCandleRequest,
  ));
}


}

/// @nodoc


class AdapterCall_Balances extends AdapterCall {
  const AdapterCall_Balances(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall_Balances);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AdapterCall.balances()';
}


}




/// @nodoc


class AdapterCall_OrderRules extends AdapterCall {
  const AdapterCall_OrderRules({required this.market}): super._();


 final  WireMarket market;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterCall_OrderRulesCopyWith<AdapterCall_OrderRules> get copyWith => _$AdapterCall_OrderRulesCopyWithImpl<AdapterCall_OrderRules>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall_OrderRules&&(identical(other.market, market) || other.market == market));
}


@override
int get hashCode => Object.hash(runtimeType,market);

@override
String toString() {
  return 'AdapterCall.orderRules(market: $market)';
}


}

/// @nodoc
abstract mixin class $AdapterCall_OrderRulesCopyWith<$Res> implements $AdapterCallCopyWith<$Res> {
  factory $AdapterCall_OrderRulesCopyWith(AdapterCall_OrderRules value, $Res Function(AdapterCall_OrderRules) _then) = _$AdapterCall_OrderRulesCopyWithImpl;
@useResult
$Res call({
 WireMarket market
});




}
/// @nodoc
class _$AdapterCall_OrderRulesCopyWithImpl<$Res>
    implements $AdapterCall_OrderRulesCopyWith<$Res> {
  _$AdapterCall_OrderRulesCopyWithImpl(this._self, this._then);

  final AdapterCall_OrderRules _self;
  final $Res Function(AdapterCall_OrderRules) _then;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? market = null,}) {
  return _then(AdapterCall_OrderRules(
market: null == market ? _self.market : market // ignore: cast_nullable_to_non_nullable
as WireMarket,
  ));
}


}

/// @nodoc


class AdapterCall_AssetNetworks extends AdapterCall {
  const AdapterCall_AssetNetworks({required this.asset}): super._();


 final  String asset;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterCall_AssetNetworksCopyWith<AdapterCall_AssetNetworks> get copyWith => _$AdapterCall_AssetNetworksCopyWithImpl<AdapterCall_AssetNetworks>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall_AssetNetworks&&(identical(other.asset, asset) || other.asset == asset));
}


@override
int get hashCode => Object.hash(runtimeType,asset);

@override
String toString() {
  return 'AdapterCall.assetNetworks(asset: $asset)';
}


}

/// @nodoc
abstract mixin class $AdapterCall_AssetNetworksCopyWith<$Res> implements $AdapterCallCopyWith<$Res> {
  factory $AdapterCall_AssetNetworksCopyWith(AdapterCall_AssetNetworks value, $Res Function(AdapterCall_AssetNetworks) _then) = _$AdapterCall_AssetNetworksCopyWithImpl;
@useResult
$Res call({
 String asset
});




}
/// @nodoc
class _$AdapterCall_AssetNetworksCopyWithImpl<$Res>
    implements $AdapterCall_AssetNetworksCopyWith<$Res> {
  _$AdapterCall_AssetNetworksCopyWithImpl(this._self, this._then);

  final AdapterCall_AssetNetworks _self;
  final $Res Function(AdapterCall_AssetNetworks) _then;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? asset = null,}) {
  return _then(AdapterCall_AssetNetworks(
asset: null == asset ? _self.asset : asset // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AdapterCall_DepositAddress extends AdapterCall {
  const AdapterCall_DepositAddress({required this.request}): super._();


 final  WireDepositAddressRequest request;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterCall_DepositAddressCopyWith<AdapterCall_DepositAddress> get copyWith => _$AdapterCall_DepositAddressCopyWithImpl<AdapterCall_DepositAddress>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall_DepositAddress&&(identical(other.request, request) || other.request == request));
}


@override
int get hashCode => Object.hash(runtimeType,request);

@override
String toString() {
  return 'AdapterCall.depositAddress(request: $request)';
}


}

/// @nodoc
abstract mixin class $AdapterCall_DepositAddressCopyWith<$Res> implements $AdapterCallCopyWith<$Res> {
  factory $AdapterCall_DepositAddressCopyWith(AdapterCall_DepositAddress value, $Res Function(AdapterCall_DepositAddress) _then) = _$AdapterCall_DepositAddressCopyWithImpl;
@useResult
$Res call({
 WireDepositAddressRequest request
});




}
/// @nodoc
class _$AdapterCall_DepositAddressCopyWithImpl<$Res>
    implements $AdapterCall_DepositAddressCopyWith<$Res> {
  _$AdapterCall_DepositAddressCopyWithImpl(this._self, this._then);

  final AdapterCall_DepositAddress _self;
  final $Res Function(AdapterCall_DepositAddress) _then;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? request = null,}) {
  return _then(AdapterCall_DepositAddress(
request: null == request ? _self.request : request // ignore: cast_nullable_to_non_nullable
as WireDepositAddressRequest,
  ));
}


}

/// @nodoc


class AdapterCall_CreateDepositAddress extends AdapterCall {
  const AdapterCall_CreateDepositAddress({required this.request}): super._();


 final  WireDepositAddressRequest request;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterCall_CreateDepositAddressCopyWith<AdapterCall_CreateDepositAddress> get copyWith => _$AdapterCall_CreateDepositAddressCopyWithImpl<AdapterCall_CreateDepositAddress>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall_CreateDepositAddress&&(identical(other.request, request) || other.request == request));
}


@override
int get hashCode => Object.hash(runtimeType,request);

@override
String toString() {
  return 'AdapterCall.createDepositAddress(request: $request)';
}


}

/// @nodoc
abstract mixin class $AdapterCall_CreateDepositAddressCopyWith<$Res> implements $AdapterCallCopyWith<$Res> {
  factory $AdapterCall_CreateDepositAddressCopyWith(AdapterCall_CreateDepositAddress value, $Res Function(AdapterCall_CreateDepositAddress) _then) = _$AdapterCall_CreateDepositAddressCopyWithImpl;
@useResult
$Res call({
 WireDepositAddressRequest request
});




}
/// @nodoc
class _$AdapterCall_CreateDepositAddressCopyWithImpl<$Res>
    implements $AdapterCall_CreateDepositAddressCopyWith<$Res> {
  _$AdapterCall_CreateDepositAddressCopyWithImpl(this._self, this._then);

  final AdapterCall_CreateDepositAddress _self;
  final $Res Function(AdapterCall_CreateDepositAddress) _then;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? request = null,}) {
  return _then(AdapterCall_CreateDepositAddress(
request: null == request ? _self.request : request // ignore: cast_nullable_to_non_nullable
as WireDepositAddressRequest,
  ));
}


}

/// @nodoc


class AdapterCall_PrepareWithdrawal extends AdapterCall {
  const AdapterCall_PrepareWithdrawal({required this.request}): super._();


 final  WireWithdrawRequest request;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterCall_PrepareWithdrawalCopyWith<AdapterCall_PrepareWithdrawal> get copyWith => _$AdapterCall_PrepareWithdrawalCopyWithImpl<AdapterCall_PrepareWithdrawal>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall_PrepareWithdrawal&&(identical(other.request, request) || other.request == request));
}


@override
int get hashCode => Object.hash(runtimeType,request);

@override
String toString() {
  return 'AdapterCall.prepareWithdrawal(request: $request)';
}


}

/// @nodoc
abstract mixin class $AdapterCall_PrepareWithdrawalCopyWith<$Res> implements $AdapterCallCopyWith<$Res> {
  factory $AdapterCall_PrepareWithdrawalCopyWith(AdapterCall_PrepareWithdrawal value, $Res Function(AdapterCall_PrepareWithdrawal) _then) = _$AdapterCall_PrepareWithdrawalCopyWithImpl;
@useResult
$Res call({
 WireWithdrawRequest request
});




}
/// @nodoc
class _$AdapterCall_PrepareWithdrawalCopyWithImpl<$Res>
    implements $AdapterCall_PrepareWithdrawalCopyWith<$Res> {
  _$AdapterCall_PrepareWithdrawalCopyWithImpl(this._self, this._then);

  final AdapterCall_PrepareWithdrawal _self;
  final $Res Function(AdapterCall_PrepareWithdrawal) _then;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? request = null,}) {
  return _then(AdapterCall_PrepareWithdrawal(
request: null == request ? _self.request : request // ignore: cast_nullable_to_non_nullable
as WireWithdrawRequest,
  ));
}


}

/// @nodoc


class AdapterCall_Withdraw extends AdapterCall {
  const AdapterCall_Withdraw({required this.request}): super._();


 final  WireWithdrawRequest request;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterCall_WithdrawCopyWith<AdapterCall_Withdraw> get copyWith => _$AdapterCall_WithdrawCopyWithImpl<AdapterCall_Withdraw>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall_Withdraw&&(identical(other.request, request) || other.request == request));
}


@override
int get hashCode => Object.hash(runtimeType,request);

@override
String toString() {
  return 'AdapterCall.withdraw(request: $request)';
}


}

/// @nodoc
abstract mixin class $AdapterCall_WithdrawCopyWith<$Res> implements $AdapterCallCopyWith<$Res> {
  factory $AdapterCall_WithdrawCopyWith(AdapterCall_Withdraw value, $Res Function(AdapterCall_Withdraw) _then) = _$AdapterCall_WithdrawCopyWithImpl;
@useResult
$Res call({
 WireWithdrawRequest request
});




}
/// @nodoc
class _$AdapterCall_WithdrawCopyWithImpl<$Res>
    implements $AdapterCall_WithdrawCopyWith<$Res> {
  _$AdapterCall_WithdrawCopyWithImpl(this._self, this._then);

  final AdapterCall_Withdraw _self;
  final $Res Function(AdapterCall_Withdraw) _then;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? request = null,}) {
  return _then(AdapterCall_Withdraw(
request: null == request ? _self.request : request // ignore: cast_nullable_to_non_nullable
as WireWithdrawRequest,
  ));
}


}

/// @nodoc


class AdapterCall_Deposits extends AdapterCall {
  const AdapterCall_Deposits({required this.request}): super._();


 final  WireTransferHistoryRequest request;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterCall_DepositsCopyWith<AdapterCall_Deposits> get copyWith => _$AdapterCall_DepositsCopyWithImpl<AdapterCall_Deposits>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall_Deposits&&(identical(other.request, request) || other.request == request));
}


@override
int get hashCode => Object.hash(runtimeType,request);

@override
String toString() {
  return 'AdapterCall.deposits(request: $request)';
}


}

/// @nodoc
abstract mixin class $AdapterCall_DepositsCopyWith<$Res> implements $AdapterCallCopyWith<$Res> {
  factory $AdapterCall_DepositsCopyWith(AdapterCall_Deposits value, $Res Function(AdapterCall_Deposits) _then) = _$AdapterCall_DepositsCopyWithImpl;
@useResult
$Res call({
 WireTransferHistoryRequest request
});




}
/// @nodoc
class _$AdapterCall_DepositsCopyWithImpl<$Res>
    implements $AdapterCall_DepositsCopyWith<$Res> {
  _$AdapterCall_DepositsCopyWithImpl(this._self, this._then);

  final AdapterCall_Deposits _self;
  final $Res Function(AdapterCall_Deposits) _then;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? request = null,}) {
  return _then(AdapterCall_Deposits(
request: null == request ? _self.request : request // ignore: cast_nullable_to_non_nullable
as WireTransferHistoryRequest,
  ));
}


}

/// @nodoc


class AdapterCall_Withdrawals extends AdapterCall {
  const AdapterCall_Withdrawals({required this.request}): super._();


 final  WireTransferHistoryRequest request;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterCall_WithdrawalsCopyWith<AdapterCall_Withdrawals> get copyWith => _$AdapterCall_WithdrawalsCopyWithImpl<AdapterCall_Withdrawals>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall_Withdrawals&&(identical(other.request, request) || other.request == request));
}


@override
int get hashCode => Object.hash(runtimeType,request);

@override
String toString() {
  return 'AdapterCall.withdrawals(request: $request)';
}


}

/// @nodoc
abstract mixin class $AdapterCall_WithdrawalsCopyWith<$Res> implements $AdapterCallCopyWith<$Res> {
  factory $AdapterCall_WithdrawalsCopyWith(AdapterCall_Withdrawals value, $Res Function(AdapterCall_Withdrawals) _then) = _$AdapterCall_WithdrawalsCopyWithImpl;
@useResult
$Res call({
 WireTransferHistoryRequest request
});




}
/// @nodoc
class _$AdapterCall_WithdrawalsCopyWithImpl<$Res>
    implements $AdapterCall_WithdrawalsCopyWith<$Res> {
  _$AdapterCall_WithdrawalsCopyWithImpl(this._self, this._then);

  final AdapterCall_Withdrawals _self;
  final $Res Function(AdapterCall_Withdrawals) _then;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? request = null,}) {
  return _then(AdapterCall_Withdrawals(
request: null == request ? _self.request : request // ignore: cast_nullable_to_non_nullable
as WireTransferHistoryRequest,
  ));
}


}

/// @nodoc


class AdapterCall_OpenOrders extends AdapterCall {
  const AdapterCall_OpenOrders({this.market}): super._();


 final  WireMarket? market;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterCall_OpenOrdersCopyWith<AdapterCall_OpenOrders> get copyWith => _$AdapterCall_OpenOrdersCopyWithImpl<AdapterCall_OpenOrders>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall_OpenOrders&&(identical(other.market, market) || other.market == market));
}


@override
int get hashCode => Object.hash(runtimeType,market);

@override
String toString() {
  return 'AdapterCall.openOrders(market: $market)';
}


}

/// @nodoc
abstract mixin class $AdapterCall_OpenOrdersCopyWith<$Res> implements $AdapterCallCopyWith<$Res> {
  factory $AdapterCall_OpenOrdersCopyWith(AdapterCall_OpenOrders value, $Res Function(AdapterCall_OpenOrders) _then) = _$AdapterCall_OpenOrdersCopyWithImpl;
@useResult
$Res call({
 WireMarket? market
});




}
/// @nodoc
class _$AdapterCall_OpenOrdersCopyWithImpl<$Res>
    implements $AdapterCall_OpenOrdersCopyWith<$Res> {
  _$AdapterCall_OpenOrdersCopyWithImpl(this._self, this._then);

  final AdapterCall_OpenOrders _self;
  final $Res Function(AdapterCall_OpenOrders) _then;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? market = freezed,}) {
  return _then(AdapterCall_OpenOrders(
market: freezed == market ? _self.market : market // ignore: cast_nullable_to_non_nullable
as WireMarket?,
  ));
}


}

/// @nodoc


class AdapterCall_Order extends AdapterCall {
  const AdapterCall_Order({required this.market, required this.orderId}): super._();


 final  WireMarket market;
 final  String orderId;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterCall_OrderCopyWith<AdapterCall_Order> get copyWith => _$AdapterCall_OrderCopyWithImpl<AdapterCall_Order>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall_Order&&(identical(other.market, market) || other.market == market)&&(identical(other.orderId, orderId) || other.orderId == orderId));
}


@override
int get hashCode => Object.hash(runtimeType,market,orderId);

@override
String toString() {
  return 'AdapterCall.order(market: $market, orderId: $orderId)';
}


}

/// @nodoc
abstract mixin class $AdapterCall_OrderCopyWith<$Res> implements $AdapterCallCopyWith<$Res> {
  factory $AdapterCall_OrderCopyWith(AdapterCall_Order value, $Res Function(AdapterCall_Order) _then) = _$AdapterCall_OrderCopyWithImpl;
@useResult
$Res call({
 WireMarket market, String orderId
});




}
/// @nodoc
class _$AdapterCall_OrderCopyWithImpl<$Res>
    implements $AdapterCall_OrderCopyWith<$Res> {
  _$AdapterCall_OrderCopyWithImpl(this._self, this._then);

  final AdapterCall_Order _self;
  final $Res Function(AdapterCall_Order) _then;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? market = null,Object? orderId = null,}) {
  return _then(AdapterCall_Order(
market: null == market ? _self.market : market // ignore: cast_nullable_to_non_nullable
as WireMarket,orderId: null == orderId ? _self.orderId : orderId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AdapterCall_OrderByClientId extends AdapterCall {
  const AdapterCall_OrderByClientId({required this.market, required this.clientId}): super._();


 final  WireMarket market;
 final  String clientId;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterCall_OrderByClientIdCopyWith<AdapterCall_OrderByClientId> get copyWith => _$AdapterCall_OrderByClientIdCopyWithImpl<AdapterCall_OrderByClientId>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall_OrderByClientId&&(identical(other.market, market) || other.market == market)&&(identical(other.clientId, clientId) || other.clientId == clientId));
}


@override
int get hashCode => Object.hash(runtimeType,market,clientId);

@override
String toString() {
  return 'AdapterCall.orderByClientId(market: $market, clientId: $clientId)';
}


}

/// @nodoc
abstract mixin class $AdapterCall_OrderByClientIdCopyWith<$Res> implements $AdapterCallCopyWith<$Res> {
  factory $AdapterCall_OrderByClientIdCopyWith(AdapterCall_OrderByClientId value, $Res Function(AdapterCall_OrderByClientId) _then) = _$AdapterCall_OrderByClientIdCopyWithImpl;
@useResult
$Res call({
 WireMarket market, String clientId
});




}
/// @nodoc
class _$AdapterCall_OrderByClientIdCopyWithImpl<$Res>
    implements $AdapterCall_OrderByClientIdCopyWith<$Res> {
  _$AdapterCall_OrderByClientIdCopyWithImpl(this._self, this._then);

  final AdapterCall_OrderByClientId _self;
  final $Res Function(AdapterCall_OrderByClientId) _then;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? market = null,Object? clientId = null,}) {
  return _then(AdapterCall_OrderByClientId(
market: null == market ? _self.market : market // ignore: cast_nullable_to_non_nullable
as WireMarket,clientId: null == clientId ? _self.clientId : clientId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AdapterCall_OrdersByIds extends AdapterCall {
  const AdapterCall_OrdersByIds({required this.request}): super._();


 final  WireOrderLookupRequest request;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterCall_OrdersByIdsCopyWith<AdapterCall_OrdersByIds> get copyWith => _$AdapterCall_OrdersByIdsCopyWithImpl<AdapterCall_OrdersByIds>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall_OrdersByIds&&(identical(other.request, request) || other.request == request));
}


@override
int get hashCode => Object.hash(runtimeType,request);

@override
String toString() {
  return 'AdapterCall.ordersByIds(request: $request)';
}


}

/// @nodoc
abstract mixin class $AdapterCall_OrdersByIdsCopyWith<$Res> implements $AdapterCallCopyWith<$Res> {
  factory $AdapterCall_OrdersByIdsCopyWith(AdapterCall_OrdersByIds value, $Res Function(AdapterCall_OrdersByIds) _then) = _$AdapterCall_OrdersByIdsCopyWithImpl;
@useResult
$Res call({
 WireOrderLookupRequest request
});




}
/// @nodoc
class _$AdapterCall_OrdersByIdsCopyWithImpl<$Res>
    implements $AdapterCall_OrdersByIdsCopyWith<$Res> {
  _$AdapterCall_OrdersByIdsCopyWithImpl(this._self, this._then);

  final AdapterCall_OrdersByIds _self;
  final $Res Function(AdapterCall_OrdersByIds) _then;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? request = null,}) {
  return _then(AdapterCall_OrdersByIds(
request: null == request ? _self.request : request // ignore: cast_nullable_to_non_nullable
as WireOrderLookupRequest,
  ));
}


}

/// @nodoc


class AdapterCall_OrderHistory extends AdapterCall {
  const AdapterCall_OrderHistory({required this.request}): super._();


 final  WireOrderHistoryRequest request;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterCall_OrderHistoryCopyWith<AdapterCall_OrderHistory> get copyWith => _$AdapterCall_OrderHistoryCopyWithImpl<AdapterCall_OrderHistory>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall_OrderHistory&&(identical(other.request, request) || other.request == request));
}


@override
int get hashCode => Object.hash(runtimeType,request);

@override
String toString() {
  return 'AdapterCall.orderHistory(request: $request)';
}


}

/// @nodoc
abstract mixin class $AdapterCall_OrderHistoryCopyWith<$Res> implements $AdapterCallCopyWith<$Res> {
  factory $AdapterCall_OrderHistoryCopyWith(AdapterCall_OrderHistory value, $Res Function(AdapterCall_OrderHistory) _then) = _$AdapterCall_OrderHistoryCopyWithImpl;
@useResult
$Res call({
 WireOrderHistoryRequest request
});




}
/// @nodoc
class _$AdapterCall_OrderHistoryCopyWithImpl<$Res>
    implements $AdapterCall_OrderHistoryCopyWith<$Res> {
  _$AdapterCall_OrderHistoryCopyWithImpl(this._self, this._then);

  final AdapterCall_OrderHistory _self;
  final $Res Function(AdapterCall_OrderHistory) _then;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? request = null,}) {
  return _then(AdapterCall_OrderHistory(
request: null == request ? _self.request : request // ignore: cast_nullable_to_non_nullable
as WireOrderHistoryRequest,
  ));
}


}

/// @nodoc


class AdapterCall_PlaceOrder extends AdapterCall {
  const AdapterCall_PlaceOrder({required this.request}): super._();


 final  WireOrderRequest request;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterCall_PlaceOrderCopyWith<AdapterCall_PlaceOrder> get copyWith => _$AdapterCall_PlaceOrderCopyWithImpl<AdapterCall_PlaceOrder>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall_PlaceOrder&&(identical(other.request, request) || other.request == request));
}


@override
int get hashCode => Object.hash(runtimeType,request);

@override
String toString() {
  return 'AdapterCall.placeOrder(request: $request)';
}


}

/// @nodoc
abstract mixin class $AdapterCall_PlaceOrderCopyWith<$Res> implements $AdapterCallCopyWith<$Res> {
  factory $AdapterCall_PlaceOrderCopyWith(AdapterCall_PlaceOrder value, $Res Function(AdapterCall_PlaceOrder) _then) = _$AdapterCall_PlaceOrderCopyWithImpl;
@useResult
$Res call({
 WireOrderRequest request
});




}
/// @nodoc
class _$AdapterCall_PlaceOrderCopyWithImpl<$Res>
    implements $AdapterCall_PlaceOrderCopyWith<$Res> {
  _$AdapterCall_PlaceOrderCopyWithImpl(this._self, this._then);

  final AdapterCall_PlaceOrder _self;
  final $Res Function(AdapterCall_PlaceOrder) _then;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? request = null,}) {
  return _then(AdapterCall_PlaceOrder(
request: null == request ? _self.request : request // ignore: cast_nullable_to_non_nullable
as WireOrderRequest,
  ));
}


}

/// @nodoc


class AdapterCall_CancelOrder extends AdapterCall {
  const AdapterCall_CancelOrder({required this.market, required this.orderId}): super._();


 final  WireMarket market;
 final  String orderId;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterCall_CancelOrderCopyWith<AdapterCall_CancelOrder> get copyWith => _$AdapterCall_CancelOrderCopyWithImpl<AdapterCall_CancelOrder>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall_CancelOrder&&(identical(other.market, market) || other.market == market)&&(identical(other.orderId, orderId) || other.orderId == orderId));
}


@override
int get hashCode => Object.hash(runtimeType,market,orderId);

@override
String toString() {
  return 'AdapterCall.cancelOrder(market: $market, orderId: $orderId)';
}


}

/// @nodoc
abstract mixin class $AdapterCall_CancelOrderCopyWith<$Res> implements $AdapterCallCopyWith<$Res> {
  factory $AdapterCall_CancelOrderCopyWith(AdapterCall_CancelOrder value, $Res Function(AdapterCall_CancelOrder) _then) = _$AdapterCall_CancelOrderCopyWithImpl;
@useResult
$Res call({
 WireMarket market, String orderId
});




}
/// @nodoc
class _$AdapterCall_CancelOrderCopyWithImpl<$Res>
    implements $AdapterCall_CancelOrderCopyWith<$Res> {
  _$AdapterCall_CancelOrderCopyWithImpl(this._self, this._then);

  final AdapterCall_CancelOrder _self;
  final $Res Function(AdapterCall_CancelOrder) _then;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? market = null,Object? orderId = null,}) {
  return _then(AdapterCall_CancelOrder(
market: null == market ? _self.market : market // ignore: cast_nullable_to_non_nullable
as WireMarket,orderId: null == orderId ? _self.orderId : orderId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AdapterCall_CancelOrderByClientId extends AdapterCall {
  const AdapterCall_CancelOrderByClientId({required this.market, required this.clientId}): super._();


 final  WireMarket market;
 final  String clientId;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterCall_CancelOrderByClientIdCopyWith<AdapterCall_CancelOrderByClientId> get copyWith => _$AdapterCall_CancelOrderByClientIdCopyWithImpl<AdapterCall_CancelOrderByClientId>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall_CancelOrderByClientId&&(identical(other.market, market) || other.market == market)&&(identical(other.clientId, clientId) || other.clientId == clientId));
}


@override
int get hashCode => Object.hash(runtimeType,market,clientId);

@override
String toString() {
  return 'AdapterCall.cancelOrderByClientId(market: $market, clientId: $clientId)';
}


}

/// @nodoc
abstract mixin class $AdapterCall_CancelOrderByClientIdCopyWith<$Res> implements $AdapterCallCopyWith<$Res> {
  factory $AdapterCall_CancelOrderByClientIdCopyWith(AdapterCall_CancelOrderByClientId value, $Res Function(AdapterCall_CancelOrderByClientId) _then) = _$AdapterCall_CancelOrderByClientIdCopyWithImpl;
@useResult
$Res call({
 WireMarket market, String clientId
});




}
/// @nodoc
class _$AdapterCall_CancelOrderByClientIdCopyWithImpl<$Res>
    implements $AdapterCall_CancelOrderByClientIdCopyWith<$Res> {
  _$AdapterCall_CancelOrderByClientIdCopyWithImpl(this._self, this._then);

  final AdapterCall_CancelOrderByClientId _self;
  final $Res Function(AdapterCall_CancelOrderByClientId) _then;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? market = null,Object? clientId = null,}) {
  return _then(AdapterCall_CancelOrderByClientId(
market: null == market ? _self.market : market // ignore: cast_nullable_to_non_nullable
as WireMarket,clientId: null == clientId ? _self.clientId : clientId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AdapterCall_CancelOrders extends AdapterCall {
  const AdapterCall_CancelOrders({required this.request}): super._();


 final  WireCancelOrdersRequest request;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterCall_CancelOrdersCopyWith<AdapterCall_CancelOrders> get copyWith => _$AdapterCall_CancelOrdersCopyWithImpl<AdapterCall_CancelOrders>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall_CancelOrders&&(identical(other.request, request) || other.request == request));
}


@override
int get hashCode => Object.hash(runtimeType,request);

@override
String toString() {
  return 'AdapterCall.cancelOrders(request: $request)';
}


}

/// @nodoc
abstract mixin class $AdapterCall_CancelOrdersCopyWith<$Res> implements $AdapterCallCopyWith<$Res> {
  factory $AdapterCall_CancelOrdersCopyWith(AdapterCall_CancelOrders value, $Res Function(AdapterCall_CancelOrders) _then) = _$AdapterCall_CancelOrdersCopyWithImpl;
@useResult
$Res call({
 WireCancelOrdersRequest request
});




}
/// @nodoc
class _$AdapterCall_CancelOrdersCopyWithImpl<$Res>
    implements $AdapterCall_CancelOrdersCopyWith<$Res> {
  _$AdapterCall_CancelOrdersCopyWithImpl(this._self, this._then);

  final AdapterCall_CancelOrders _self;
  final $Res Function(AdapterCall_CancelOrders) _then;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? request = null,}) {
  return _then(AdapterCall_CancelOrders(
request: null == request ? _self.request : request // ignore: cast_nullable_to_non_nullable
as WireCancelOrdersRequest,
  ));
}


}

/// @nodoc


class AdapterCall_Positions extends AdapterCall {
  const AdapterCall_Positions({this.market}): super._();


 final  WireMarket? market;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterCall_PositionsCopyWith<AdapterCall_Positions> get copyWith => _$AdapterCall_PositionsCopyWithImpl<AdapterCall_Positions>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall_Positions&&(identical(other.market, market) || other.market == market));
}


@override
int get hashCode => Object.hash(runtimeType,market);

@override
String toString() {
  return 'AdapterCall.positions(market: $market)';
}


}

/// @nodoc
abstract mixin class $AdapterCall_PositionsCopyWith<$Res> implements $AdapterCallCopyWith<$Res> {
  factory $AdapterCall_PositionsCopyWith(AdapterCall_Positions value, $Res Function(AdapterCall_Positions) _then) = _$AdapterCall_PositionsCopyWithImpl;
@useResult
$Res call({
 WireMarket? market
});




}
/// @nodoc
class _$AdapterCall_PositionsCopyWithImpl<$Res>
    implements $AdapterCall_PositionsCopyWith<$Res> {
  _$AdapterCall_PositionsCopyWithImpl(this._self, this._then);

  final AdapterCall_Positions _self;
  final $Res Function(AdapterCall_Positions) _then;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? market = freezed,}) {
  return _then(AdapterCall_Positions(
market: freezed == market ? _self.market : market // ignore: cast_nullable_to_non_nullable
as WireMarket?,
  ));
}


}

/// @nodoc


class AdapterCall_MarginSummary extends AdapterCall {
  const AdapterCall_MarginSummary(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall_MarginSummary);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AdapterCall.marginSummary()';
}


}




/// @nodoc


class AdapterCall_FundingRates extends AdapterCall {
  const AdapterCall_FundingRates({required this.request}): super._();


 final  WireHistoryRequest request;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterCall_FundingRatesCopyWith<AdapterCall_FundingRates> get copyWith => _$AdapterCall_FundingRatesCopyWithImpl<AdapterCall_FundingRates>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall_FundingRates&&(identical(other.request, request) || other.request == request));
}


@override
int get hashCode => Object.hash(runtimeType,request);

@override
String toString() {
  return 'AdapterCall.fundingRates(request: $request)';
}


}

/// @nodoc
abstract mixin class $AdapterCall_FundingRatesCopyWith<$Res> implements $AdapterCallCopyWith<$Res> {
  factory $AdapterCall_FundingRatesCopyWith(AdapterCall_FundingRates value, $Res Function(AdapterCall_FundingRates) _then) = _$AdapterCall_FundingRatesCopyWithImpl;
@useResult
$Res call({
 WireHistoryRequest request
});




}
/// @nodoc
class _$AdapterCall_FundingRatesCopyWithImpl<$Res>
    implements $AdapterCall_FundingRatesCopyWith<$Res> {
  _$AdapterCall_FundingRatesCopyWithImpl(this._self, this._then);

  final AdapterCall_FundingRates _self;
  final $Res Function(AdapterCall_FundingRates) _then;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? request = null,}) {
  return _then(AdapterCall_FundingRates(
request: null == request ? _self.request : request // ignore: cast_nullable_to_non_nullable
as WireHistoryRequest,
  ));
}


}

/// @nodoc


class AdapterCall_FundingPayments extends AdapterCall {
  const AdapterCall_FundingPayments({required this.request}): super._();


 final  WireHistoryRequest request;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterCall_FundingPaymentsCopyWith<AdapterCall_FundingPayments> get copyWith => _$AdapterCall_FundingPaymentsCopyWithImpl<AdapterCall_FundingPayments>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall_FundingPayments&&(identical(other.request, request) || other.request == request));
}


@override
int get hashCode => Object.hash(runtimeType,request);

@override
String toString() {
  return 'AdapterCall.fundingPayments(request: $request)';
}


}

/// @nodoc
abstract mixin class $AdapterCall_FundingPaymentsCopyWith<$Res> implements $AdapterCallCopyWith<$Res> {
  factory $AdapterCall_FundingPaymentsCopyWith(AdapterCall_FundingPayments value, $Res Function(AdapterCall_FundingPayments) _then) = _$AdapterCall_FundingPaymentsCopyWithImpl;
@useResult
$Res call({
 WireHistoryRequest request
});




}
/// @nodoc
class _$AdapterCall_FundingPaymentsCopyWithImpl<$Res>
    implements $AdapterCall_FundingPaymentsCopyWith<$Res> {
  _$AdapterCall_FundingPaymentsCopyWithImpl(this._self, this._then);

  final AdapterCall_FundingPayments _self;
  final $Res Function(AdapterCall_FundingPayments) _then;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? request = null,}) {
  return _then(AdapterCall_FundingPayments(
request: null == request ? _self.request : request // ignore: cast_nullable_to_non_nullable
as WireHistoryRequest,
  ));
}


}

/// @nodoc


class AdapterCall_SetMargin extends AdapterCall {
  const AdapterCall_SetMargin({required this.request}): super._();


 final  WireMarginRequest request;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterCall_SetMarginCopyWith<AdapterCall_SetMargin> get copyWith => _$AdapterCall_SetMarginCopyWithImpl<AdapterCall_SetMargin>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall_SetMargin&&(identical(other.request, request) || other.request == request));
}


@override
int get hashCode => Object.hash(runtimeType,request);

@override
String toString() {
  return 'AdapterCall.setMargin(request: $request)';
}


}

/// @nodoc
abstract mixin class $AdapterCall_SetMarginCopyWith<$Res> implements $AdapterCallCopyWith<$Res> {
  factory $AdapterCall_SetMarginCopyWith(AdapterCall_SetMargin value, $Res Function(AdapterCall_SetMargin) _then) = _$AdapterCall_SetMarginCopyWithImpl;
@useResult
$Res call({
 WireMarginRequest request
});




}
/// @nodoc
class _$AdapterCall_SetMarginCopyWithImpl<$Res>
    implements $AdapterCall_SetMarginCopyWith<$Res> {
  _$AdapterCall_SetMarginCopyWithImpl(this._self, this._then);

  final AdapterCall_SetMargin _self;
  final $Res Function(AdapterCall_SetMargin) _then;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? request = null,}) {
  return _then(AdapterCall_SetMargin(
request: null == request ? _self.request : request // ignore: cast_nullable_to_non_nullable
as WireMarginRequest,
  ));
}


}

/// @nodoc


class AdapterCall_Subscribe extends AdapterCall {
  const AdapterCall_Subscribe({required this.streamId, required this.subscription, required this.config, required this.sink}): super._();


 final  String streamId;
 final  WireSubscription subscription;
 final  WireStreamConfig config;
 final  MarketStreamSink sink;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterCall_SubscribeCopyWith<AdapterCall_Subscribe> get copyWith => _$AdapterCall_SubscribeCopyWithImpl<AdapterCall_Subscribe>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall_Subscribe&&(identical(other.streamId, streamId) || other.streamId == streamId)&&(identical(other.subscription, subscription) || other.subscription == subscription)&&(identical(other.config, config) || other.config == config)&&(identical(other.sink, sink) || other.sink == sink));
}


@override
int get hashCode => Object.hash(runtimeType,streamId,subscription,config,sink);

@override
String toString() {
  return 'AdapterCall.subscribe(streamId: $streamId, subscription: $subscription, config: $config, sink: $sink)';
}


}

/// @nodoc
abstract mixin class $AdapterCall_SubscribeCopyWith<$Res> implements $AdapterCallCopyWith<$Res> {
  factory $AdapterCall_SubscribeCopyWith(AdapterCall_Subscribe value, $Res Function(AdapterCall_Subscribe) _then) = _$AdapterCall_SubscribeCopyWithImpl;
@useResult
$Res call({
 String streamId, WireSubscription subscription, WireStreamConfig config, MarketStreamSink sink
});




}
/// @nodoc
class _$AdapterCall_SubscribeCopyWithImpl<$Res>
    implements $AdapterCall_SubscribeCopyWith<$Res> {
  _$AdapterCall_SubscribeCopyWithImpl(this._self, this._then);

  final AdapterCall_Subscribe _self;
  final $Res Function(AdapterCall_Subscribe) _then;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? streamId = null,Object? subscription = null,Object? config = null,Object? sink = null,}) {
  return _then(AdapterCall_Subscribe(
streamId: null == streamId ? _self.streamId : streamId // ignore: cast_nullable_to_non_nullable
as String,subscription: null == subscription ? _self.subscription : subscription // ignore: cast_nullable_to_non_nullable
as WireSubscription,config: null == config ? _self.config : config // ignore: cast_nullable_to_non_nullable
as WireStreamConfig,sink: null == sink ? _self.sink : sink // ignore: cast_nullable_to_non_nullable
as MarketStreamSink,
  ));
}


}

/// @nodoc


class AdapterCall_SubscribeAccount extends AdapterCall {
  const AdapterCall_SubscribeAccount({required this.streamId, required this.config, required this.sink}): super._();


 final  String streamId;
 final  WireStreamConfig config;
 final  AccountStreamSink sink;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterCall_SubscribeAccountCopyWith<AdapterCall_SubscribeAccount> get copyWith => _$AdapterCall_SubscribeAccountCopyWithImpl<AdapterCall_SubscribeAccount>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall_SubscribeAccount&&(identical(other.streamId, streamId) || other.streamId == streamId)&&(identical(other.config, config) || other.config == config)&&(identical(other.sink, sink) || other.sink == sink));
}


@override
int get hashCode => Object.hash(runtimeType,streamId,config,sink);

@override
String toString() {
  return 'AdapterCall.subscribeAccount(streamId: $streamId, config: $config, sink: $sink)';
}


}

/// @nodoc
abstract mixin class $AdapterCall_SubscribeAccountCopyWith<$Res> implements $AdapterCallCopyWith<$Res> {
  factory $AdapterCall_SubscribeAccountCopyWith(AdapterCall_SubscribeAccount value, $Res Function(AdapterCall_SubscribeAccount) _then) = _$AdapterCall_SubscribeAccountCopyWithImpl;
@useResult
$Res call({
 String streamId, WireStreamConfig config, AccountStreamSink sink
});




}
/// @nodoc
class _$AdapterCall_SubscribeAccountCopyWithImpl<$Res>
    implements $AdapterCall_SubscribeAccountCopyWith<$Res> {
  _$AdapterCall_SubscribeAccountCopyWithImpl(this._self, this._then);

  final AdapterCall_SubscribeAccount _self;
  final $Res Function(AdapterCall_SubscribeAccount) _then;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? streamId = null,Object? config = null,Object? sink = null,}) {
  return _then(AdapterCall_SubscribeAccount(
streamId: null == streamId ? _self.streamId : streamId // ignore: cast_nullable_to_non_nullable
as String,config: null == config ? _self.config : config // ignore: cast_nullable_to_non_nullable
as WireStreamConfig,sink: null == sink ? _self.sink : sink // ignore: cast_nullable_to_non_nullable
as AccountStreamSink,
  ));
}


}

/// @nodoc


class AdapterCall_CancelStream extends AdapterCall {
  const AdapterCall_CancelStream({required this.streamId}): super._();


 final  String streamId;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterCall_CancelStreamCopyWith<AdapterCall_CancelStream> get copyWith => _$AdapterCall_CancelStreamCopyWithImpl<AdapterCall_CancelStream>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterCall_CancelStream&&(identical(other.streamId, streamId) || other.streamId == streamId));
}


@override
int get hashCode => Object.hash(runtimeType,streamId);

@override
String toString() {
  return 'AdapterCall.cancelStream(streamId: $streamId)';
}


}

/// @nodoc
abstract mixin class $AdapterCall_CancelStreamCopyWith<$Res> implements $AdapterCallCopyWith<$Res> {
  factory $AdapterCall_CancelStreamCopyWith(AdapterCall_CancelStream value, $Res Function(AdapterCall_CancelStream) _then) = _$AdapterCall_CancelStreamCopyWithImpl;
@useResult
$Res call({
 String streamId
});




}
/// @nodoc
class _$AdapterCall_CancelStreamCopyWithImpl<$Res>
    implements $AdapterCall_CancelStreamCopyWith<$Res> {
  _$AdapterCall_CancelStreamCopyWithImpl(this._self, this._then);

  final AdapterCall_CancelStream _self;
  final $Res Function(AdapterCall_CancelStream) _then;

/// Create a copy of AdapterCall
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? streamId = null,}) {
  return _then(AdapterCall_CancelStream(
streamId: null == streamId ? _self.streamId : streamId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$AdapterReply {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterReply);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AdapterReply()';
}


}

/// @nodoc
class $AdapterReplyCopyWith<$Res>  {
$AdapterReplyCopyWith(AdapterReply _, $Res Function(AdapterReply) __);
}


/// Adds pattern-matching-related methods to [AdapterReply].
extension AdapterReplyPatterns on AdapterReply {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( AdapterReply_Markets value)?  markets,TResult Function( AdapterReply_Trades value)?  trades,TResult Function( AdapterReply_OrderBook value)?  orderBook,TResult Function( AdapterReply_Ticker value)?  ticker,TResult Function( AdapterReply_Candles value)?  candles,TResult Function( AdapterReply_Balances value)?  balances,TResult Function( AdapterReply_OrderRules value)?  orderRules,TResult Function( AdapterReply_AssetNetworks value)?  assetNetworks,TResult Function( AdapterReply_DepositAddress value)?  depositAddress,TResult Function( AdapterReply_CreateDepositAddress value)?  createDepositAddress,TResult Function( AdapterReply_PrepareWithdrawal value)?  prepareWithdrawal,TResult Function( AdapterReply_Withdraw value)?  withdraw,TResult Function( AdapterReply_Deposits value)?  deposits,TResult Function( AdapterReply_Withdrawals value)?  withdrawals,TResult Function( AdapterReply_OpenOrders value)?  openOrders,TResult Function( AdapterReply_Order value)?  order,TResult Function( AdapterReply_OrdersByIds value)?  ordersByIds,TResult Function( AdapterReply_OrderHistory value)?  orderHistory,TResult Function( AdapterReply_PlaceOrder value)?  placeOrder,TResult Function( AdapterReply_CancelOrders value)?  cancelOrders,TResult Function( AdapterReply_Positions value)?  positions,TResult Function( AdapterReply_MarginSummary value)?  marginSummary,TResult Function( AdapterReply_FundingRates value)?  fundingRates,TResult Function( AdapterReply_FundingPayments value)?  fundingPayments,TResult Function( AdapterReply_Unit value)?  unit,required TResult orElse(),}){
final _that = this;
switch (_that) {
case AdapterReply_Markets() when markets != null:
return markets(_that);case AdapterReply_Trades() when trades != null:
return trades(_that);case AdapterReply_OrderBook() when orderBook != null:
return orderBook(_that);case AdapterReply_Ticker() when ticker != null:
return ticker(_that);case AdapterReply_Candles() when candles != null:
return candles(_that);case AdapterReply_Balances() when balances != null:
return balances(_that);case AdapterReply_OrderRules() when orderRules != null:
return orderRules(_that);case AdapterReply_AssetNetworks() when assetNetworks != null:
return assetNetworks(_that);case AdapterReply_DepositAddress() when depositAddress != null:
return depositAddress(_that);case AdapterReply_CreateDepositAddress() when createDepositAddress != null:
return createDepositAddress(_that);case AdapterReply_PrepareWithdrawal() when prepareWithdrawal != null:
return prepareWithdrawal(_that);case AdapterReply_Withdraw() when withdraw != null:
return withdraw(_that);case AdapterReply_Deposits() when deposits != null:
return deposits(_that);case AdapterReply_Withdrawals() when withdrawals != null:
return withdrawals(_that);case AdapterReply_OpenOrders() when openOrders != null:
return openOrders(_that);case AdapterReply_Order() when order != null:
return order(_that);case AdapterReply_OrdersByIds() when ordersByIds != null:
return ordersByIds(_that);case AdapterReply_OrderHistory() when orderHistory != null:
return orderHistory(_that);case AdapterReply_PlaceOrder() when placeOrder != null:
return placeOrder(_that);case AdapterReply_CancelOrders() when cancelOrders != null:
return cancelOrders(_that);case AdapterReply_Positions() when positions != null:
return positions(_that);case AdapterReply_MarginSummary() when marginSummary != null:
return marginSummary(_that);case AdapterReply_FundingRates() when fundingRates != null:
return fundingRates(_that);case AdapterReply_FundingPayments() when fundingPayments != null:
return fundingPayments(_that);case AdapterReply_Unit() when unit != null:
return unit(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( AdapterReply_Markets value)  markets,required TResult Function( AdapterReply_Trades value)  trades,required TResult Function( AdapterReply_OrderBook value)  orderBook,required TResult Function( AdapterReply_Ticker value)  ticker,required TResult Function( AdapterReply_Candles value)  candles,required TResult Function( AdapterReply_Balances value)  balances,required TResult Function( AdapterReply_OrderRules value)  orderRules,required TResult Function( AdapterReply_AssetNetworks value)  assetNetworks,required TResult Function( AdapterReply_DepositAddress value)  depositAddress,required TResult Function( AdapterReply_CreateDepositAddress value)  createDepositAddress,required TResult Function( AdapterReply_PrepareWithdrawal value)  prepareWithdrawal,required TResult Function( AdapterReply_Withdraw value)  withdraw,required TResult Function( AdapterReply_Deposits value)  deposits,required TResult Function( AdapterReply_Withdrawals value)  withdrawals,required TResult Function( AdapterReply_OpenOrders value)  openOrders,required TResult Function( AdapterReply_Order value)  order,required TResult Function( AdapterReply_OrdersByIds value)  ordersByIds,required TResult Function( AdapterReply_OrderHistory value)  orderHistory,required TResult Function( AdapterReply_PlaceOrder value)  placeOrder,required TResult Function( AdapterReply_CancelOrders value)  cancelOrders,required TResult Function( AdapterReply_Positions value)  positions,required TResult Function( AdapterReply_MarginSummary value)  marginSummary,required TResult Function( AdapterReply_FundingRates value)  fundingRates,required TResult Function( AdapterReply_FundingPayments value)  fundingPayments,required TResult Function( AdapterReply_Unit value)  unit,}){
final _that = this;
switch (_that) {
case AdapterReply_Markets():
return markets(_that);case AdapterReply_Trades():
return trades(_that);case AdapterReply_OrderBook():
return orderBook(_that);case AdapterReply_Ticker():
return ticker(_that);case AdapterReply_Candles():
return candles(_that);case AdapterReply_Balances():
return balances(_that);case AdapterReply_OrderRules():
return orderRules(_that);case AdapterReply_AssetNetworks():
return assetNetworks(_that);case AdapterReply_DepositAddress():
return depositAddress(_that);case AdapterReply_CreateDepositAddress():
return createDepositAddress(_that);case AdapterReply_PrepareWithdrawal():
return prepareWithdrawal(_that);case AdapterReply_Withdraw():
return withdraw(_that);case AdapterReply_Deposits():
return deposits(_that);case AdapterReply_Withdrawals():
return withdrawals(_that);case AdapterReply_OpenOrders():
return openOrders(_that);case AdapterReply_Order():
return order(_that);case AdapterReply_OrdersByIds():
return ordersByIds(_that);case AdapterReply_OrderHistory():
return orderHistory(_that);case AdapterReply_PlaceOrder():
return placeOrder(_that);case AdapterReply_CancelOrders():
return cancelOrders(_that);case AdapterReply_Positions():
return positions(_that);case AdapterReply_MarginSummary():
return marginSummary(_that);case AdapterReply_FundingRates():
return fundingRates(_that);case AdapterReply_FundingPayments():
return fundingPayments(_that);case AdapterReply_Unit():
return unit(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( AdapterReply_Markets value)?  markets,TResult? Function( AdapterReply_Trades value)?  trades,TResult? Function( AdapterReply_OrderBook value)?  orderBook,TResult? Function( AdapterReply_Ticker value)?  ticker,TResult? Function( AdapterReply_Candles value)?  candles,TResult? Function( AdapterReply_Balances value)?  balances,TResult? Function( AdapterReply_OrderRules value)?  orderRules,TResult? Function( AdapterReply_AssetNetworks value)?  assetNetworks,TResult? Function( AdapterReply_DepositAddress value)?  depositAddress,TResult? Function( AdapterReply_CreateDepositAddress value)?  createDepositAddress,TResult? Function( AdapterReply_PrepareWithdrawal value)?  prepareWithdrawal,TResult? Function( AdapterReply_Withdraw value)?  withdraw,TResult? Function( AdapterReply_Deposits value)?  deposits,TResult? Function( AdapterReply_Withdrawals value)?  withdrawals,TResult? Function( AdapterReply_OpenOrders value)?  openOrders,TResult? Function( AdapterReply_Order value)?  order,TResult? Function( AdapterReply_OrdersByIds value)?  ordersByIds,TResult? Function( AdapterReply_OrderHistory value)?  orderHistory,TResult? Function( AdapterReply_PlaceOrder value)?  placeOrder,TResult? Function( AdapterReply_CancelOrders value)?  cancelOrders,TResult? Function( AdapterReply_Positions value)?  positions,TResult? Function( AdapterReply_MarginSummary value)?  marginSummary,TResult? Function( AdapterReply_FundingRates value)?  fundingRates,TResult? Function( AdapterReply_FundingPayments value)?  fundingPayments,TResult? Function( AdapterReply_Unit value)?  unit,}){
final _that = this;
switch (_that) {
case AdapterReply_Markets() when markets != null:
return markets(_that);case AdapterReply_Trades() when trades != null:
return trades(_that);case AdapterReply_OrderBook() when orderBook != null:
return orderBook(_that);case AdapterReply_Ticker() when ticker != null:
return ticker(_that);case AdapterReply_Candles() when candles != null:
return candles(_that);case AdapterReply_Balances() when balances != null:
return balances(_that);case AdapterReply_OrderRules() when orderRules != null:
return orderRules(_that);case AdapterReply_AssetNetworks() when assetNetworks != null:
return assetNetworks(_that);case AdapterReply_DepositAddress() when depositAddress != null:
return depositAddress(_that);case AdapterReply_CreateDepositAddress() when createDepositAddress != null:
return createDepositAddress(_that);case AdapterReply_PrepareWithdrawal() when prepareWithdrawal != null:
return prepareWithdrawal(_that);case AdapterReply_Withdraw() when withdraw != null:
return withdraw(_that);case AdapterReply_Deposits() when deposits != null:
return deposits(_that);case AdapterReply_Withdrawals() when withdrawals != null:
return withdrawals(_that);case AdapterReply_OpenOrders() when openOrders != null:
return openOrders(_that);case AdapterReply_Order() when order != null:
return order(_that);case AdapterReply_OrdersByIds() when ordersByIds != null:
return ordersByIds(_that);case AdapterReply_OrderHistory() when orderHistory != null:
return orderHistory(_that);case AdapterReply_PlaceOrder() when placeOrder != null:
return placeOrder(_that);case AdapterReply_CancelOrders() when cancelOrders != null:
return cancelOrders(_that);case AdapterReply_Positions() when positions != null:
return positions(_that);case AdapterReply_MarginSummary() when marginSummary != null:
return marginSummary(_that);case AdapterReply_FundingRates() when fundingRates != null:
return fundingRates(_that);case AdapterReply_FundingPayments() when fundingPayments != null:
return fundingPayments(_that);case AdapterReply_Unit() when unit != null:
return unit(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( List<WireMarketInfo> field0)?  markets,TResult Function( List<WireTrade> field0)?  trades,TResult Function( WireOrderBook field0)?  orderBook,TResult Function( WireTicker field0)?  ticker,TResult Function( List<WireCandle> field0)?  candles,TResult Function( List<WireBalance> field0)?  balances,TResult Function( WireOrderRules field0)?  orderRules,TResult Function( List<WireAssetNetwork> field0)?  assetNetworks,TResult Function( WireDepositAddress field0)?  depositAddress,TResult Function( WireDepositAddress field0)?  createDepositAddress,TResult Function( WireWithdrawalQuote field0)?  prepareWithdrawal,TResult Function( WireWithdrawal field0)?  withdraw,TResult Function( WireDepositPage field0)?  deposits,TResult Function( WireWithdrawalPage field0)?  withdrawals,TResult Function( List<WireOrder> field0)?  openOrders,TResult Function( WireOrder field0)?  order,TResult Function( List<WireOrder> field0)?  ordersByIds,TResult Function( WireOrderPage field0)?  orderHistory,TResult Function( WireOrder field0)?  placeOrder,TResult Function( WireCancelOrdersResult field0)?  cancelOrders,TResult Function( List<WirePosition> field0)?  positions,TResult Function( WireMarginSummary field0)?  marginSummary,TResult Function( WireFundingRatePage field0)?  fundingRates,TResult Function( WireFundingPaymentPage field0)?  fundingPayments,TResult Function()?  unit,required TResult orElse(),}) {final _that = this;
switch (_that) {
case AdapterReply_Markets() when markets != null:
return markets(_that.field0);case AdapterReply_Trades() when trades != null:
return trades(_that.field0);case AdapterReply_OrderBook() when orderBook != null:
return orderBook(_that.field0);case AdapterReply_Ticker() when ticker != null:
return ticker(_that.field0);case AdapterReply_Candles() when candles != null:
return candles(_that.field0);case AdapterReply_Balances() when balances != null:
return balances(_that.field0);case AdapterReply_OrderRules() when orderRules != null:
return orderRules(_that.field0);case AdapterReply_AssetNetworks() when assetNetworks != null:
return assetNetworks(_that.field0);case AdapterReply_DepositAddress() when depositAddress != null:
return depositAddress(_that.field0);case AdapterReply_CreateDepositAddress() when createDepositAddress != null:
return createDepositAddress(_that.field0);case AdapterReply_PrepareWithdrawal() when prepareWithdrawal != null:
return prepareWithdrawal(_that.field0);case AdapterReply_Withdraw() when withdraw != null:
return withdraw(_that.field0);case AdapterReply_Deposits() when deposits != null:
return deposits(_that.field0);case AdapterReply_Withdrawals() when withdrawals != null:
return withdrawals(_that.field0);case AdapterReply_OpenOrders() when openOrders != null:
return openOrders(_that.field0);case AdapterReply_Order() when order != null:
return order(_that.field0);case AdapterReply_OrdersByIds() when ordersByIds != null:
return ordersByIds(_that.field0);case AdapterReply_OrderHistory() when orderHistory != null:
return orderHistory(_that.field0);case AdapterReply_PlaceOrder() when placeOrder != null:
return placeOrder(_that.field0);case AdapterReply_CancelOrders() when cancelOrders != null:
return cancelOrders(_that.field0);case AdapterReply_Positions() when positions != null:
return positions(_that.field0);case AdapterReply_MarginSummary() when marginSummary != null:
return marginSummary(_that.field0);case AdapterReply_FundingRates() when fundingRates != null:
return fundingRates(_that.field0);case AdapterReply_FundingPayments() when fundingPayments != null:
return fundingPayments(_that.field0);case AdapterReply_Unit() when unit != null:
return unit();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( List<WireMarketInfo> field0)  markets,required TResult Function( List<WireTrade> field0)  trades,required TResult Function( WireOrderBook field0)  orderBook,required TResult Function( WireTicker field0)  ticker,required TResult Function( List<WireCandle> field0)  candles,required TResult Function( List<WireBalance> field0)  balances,required TResult Function( WireOrderRules field0)  orderRules,required TResult Function( List<WireAssetNetwork> field0)  assetNetworks,required TResult Function( WireDepositAddress field0)  depositAddress,required TResult Function( WireDepositAddress field0)  createDepositAddress,required TResult Function( WireWithdrawalQuote field0)  prepareWithdrawal,required TResult Function( WireWithdrawal field0)  withdraw,required TResult Function( WireDepositPage field0)  deposits,required TResult Function( WireWithdrawalPage field0)  withdrawals,required TResult Function( List<WireOrder> field0)  openOrders,required TResult Function( WireOrder field0)  order,required TResult Function( List<WireOrder> field0)  ordersByIds,required TResult Function( WireOrderPage field0)  orderHistory,required TResult Function( WireOrder field0)  placeOrder,required TResult Function( WireCancelOrdersResult field0)  cancelOrders,required TResult Function( List<WirePosition> field0)  positions,required TResult Function( WireMarginSummary field0)  marginSummary,required TResult Function( WireFundingRatePage field0)  fundingRates,required TResult Function( WireFundingPaymentPage field0)  fundingPayments,required TResult Function()  unit,}) {final _that = this;
switch (_that) {
case AdapterReply_Markets():
return markets(_that.field0);case AdapterReply_Trades():
return trades(_that.field0);case AdapterReply_OrderBook():
return orderBook(_that.field0);case AdapterReply_Ticker():
return ticker(_that.field0);case AdapterReply_Candles():
return candles(_that.field0);case AdapterReply_Balances():
return balances(_that.field0);case AdapterReply_OrderRules():
return orderRules(_that.field0);case AdapterReply_AssetNetworks():
return assetNetworks(_that.field0);case AdapterReply_DepositAddress():
return depositAddress(_that.field0);case AdapterReply_CreateDepositAddress():
return createDepositAddress(_that.field0);case AdapterReply_PrepareWithdrawal():
return prepareWithdrawal(_that.field0);case AdapterReply_Withdraw():
return withdraw(_that.field0);case AdapterReply_Deposits():
return deposits(_that.field0);case AdapterReply_Withdrawals():
return withdrawals(_that.field0);case AdapterReply_OpenOrders():
return openOrders(_that.field0);case AdapterReply_Order():
return order(_that.field0);case AdapterReply_OrdersByIds():
return ordersByIds(_that.field0);case AdapterReply_OrderHistory():
return orderHistory(_that.field0);case AdapterReply_PlaceOrder():
return placeOrder(_that.field0);case AdapterReply_CancelOrders():
return cancelOrders(_that.field0);case AdapterReply_Positions():
return positions(_that.field0);case AdapterReply_MarginSummary():
return marginSummary(_that.field0);case AdapterReply_FundingRates():
return fundingRates(_that.field0);case AdapterReply_FundingPayments():
return fundingPayments(_that.field0);case AdapterReply_Unit():
return unit();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( List<WireMarketInfo> field0)?  markets,TResult? Function( List<WireTrade> field0)?  trades,TResult? Function( WireOrderBook field0)?  orderBook,TResult? Function( WireTicker field0)?  ticker,TResult? Function( List<WireCandle> field0)?  candles,TResult? Function( List<WireBalance> field0)?  balances,TResult? Function( WireOrderRules field0)?  orderRules,TResult? Function( List<WireAssetNetwork> field0)?  assetNetworks,TResult? Function( WireDepositAddress field0)?  depositAddress,TResult? Function( WireDepositAddress field0)?  createDepositAddress,TResult? Function( WireWithdrawalQuote field0)?  prepareWithdrawal,TResult? Function( WireWithdrawal field0)?  withdraw,TResult? Function( WireDepositPage field0)?  deposits,TResult? Function( WireWithdrawalPage field0)?  withdrawals,TResult? Function( List<WireOrder> field0)?  openOrders,TResult? Function( WireOrder field0)?  order,TResult? Function( List<WireOrder> field0)?  ordersByIds,TResult? Function( WireOrderPage field0)?  orderHistory,TResult? Function( WireOrder field0)?  placeOrder,TResult? Function( WireCancelOrdersResult field0)?  cancelOrders,TResult? Function( List<WirePosition> field0)?  positions,TResult? Function( WireMarginSummary field0)?  marginSummary,TResult? Function( WireFundingRatePage field0)?  fundingRates,TResult? Function( WireFundingPaymentPage field0)?  fundingPayments,TResult? Function()?  unit,}) {final _that = this;
switch (_that) {
case AdapterReply_Markets() when markets != null:
return markets(_that.field0);case AdapterReply_Trades() when trades != null:
return trades(_that.field0);case AdapterReply_OrderBook() when orderBook != null:
return orderBook(_that.field0);case AdapterReply_Ticker() when ticker != null:
return ticker(_that.field0);case AdapterReply_Candles() when candles != null:
return candles(_that.field0);case AdapterReply_Balances() when balances != null:
return balances(_that.field0);case AdapterReply_OrderRules() when orderRules != null:
return orderRules(_that.field0);case AdapterReply_AssetNetworks() when assetNetworks != null:
return assetNetworks(_that.field0);case AdapterReply_DepositAddress() when depositAddress != null:
return depositAddress(_that.field0);case AdapterReply_CreateDepositAddress() when createDepositAddress != null:
return createDepositAddress(_that.field0);case AdapterReply_PrepareWithdrawal() when prepareWithdrawal != null:
return prepareWithdrawal(_that.field0);case AdapterReply_Withdraw() when withdraw != null:
return withdraw(_that.field0);case AdapterReply_Deposits() when deposits != null:
return deposits(_that.field0);case AdapterReply_Withdrawals() when withdrawals != null:
return withdrawals(_that.field0);case AdapterReply_OpenOrders() when openOrders != null:
return openOrders(_that.field0);case AdapterReply_Order() when order != null:
return order(_that.field0);case AdapterReply_OrdersByIds() when ordersByIds != null:
return ordersByIds(_that.field0);case AdapterReply_OrderHistory() when orderHistory != null:
return orderHistory(_that.field0);case AdapterReply_PlaceOrder() when placeOrder != null:
return placeOrder(_that.field0);case AdapterReply_CancelOrders() when cancelOrders != null:
return cancelOrders(_that.field0);case AdapterReply_Positions() when positions != null:
return positions(_that.field0);case AdapterReply_MarginSummary() when marginSummary != null:
return marginSummary(_that.field0);case AdapterReply_FundingRates() when fundingRates != null:
return fundingRates(_that.field0);case AdapterReply_FundingPayments() when fundingPayments != null:
return fundingPayments(_that.field0);case AdapterReply_Unit() when unit != null:
return unit();case _:
  return null;

}
}

}

/// @nodoc


class AdapterReply_Markets extends AdapterReply {
  const AdapterReply_Markets(final  List<WireMarketInfo> field0): _field0 = field0,super._();


 final  List<WireMarketInfo> _field0;
 List<WireMarketInfo> get field0 {
  if (_field0 is EqualUnmodifiableListView) return _field0;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_field0);
}


/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterReply_MarketsCopyWith<AdapterReply_Markets> get copyWith => _$AdapterReply_MarketsCopyWithImpl<AdapterReply_Markets>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterReply_Markets&&const DeepCollectionEquality().equals(other._field0, _field0));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_field0));

@override
String toString() {
  return 'AdapterReply.markets(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AdapterReply_MarketsCopyWith<$Res> implements $AdapterReplyCopyWith<$Res> {
  factory $AdapterReply_MarketsCopyWith(AdapterReply_Markets value, $Res Function(AdapterReply_Markets) _then) = _$AdapterReply_MarketsCopyWithImpl;
@useResult
$Res call({
 List<WireMarketInfo> field0
});




}
/// @nodoc
class _$AdapterReply_MarketsCopyWithImpl<$Res>
    implements $AdapterReply_MarketsCopyWith<$Res> {
  _$AdapterReply_MarketsCopyWithImpl(this._self, this._then);

  final AdapterReply_Markets _self;
  final $Res Function(AdapterReply_Markets) _then;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AdapterReply_Markets(
null == field0 ? _self._field0 : field0 // ignore: cast_nullable_to_non_nullable
as List<WireMarketInfo>,
  ));
}


}

/// @nodoc


class AdapterReply_Trades extends AdapterReply {
  const AdapterReply_Trades(final  List<WireTrade> field0): _field0 = field0,super._();


 final  List<WireTrade> _field0;
 List<WireTrade> get field0 {
  if (_field0 is EqualUnmodifiableListView) return _field0;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_field0);
}


/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterReply_TradesCopyWith<AdapterReply_Trades> get copyWith => _$AdapterReply_TradesCopyWithImpl<AdapterReply_Trades>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterReply_Trades&&const DeepCollectionEquality().equals(other._field0, _field0));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_field0));

@override
String toString() {
  return 'AdapterReply.trades(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AdapterReply_TradesCopyWith<$Res> implements $AdapterReplyCopyWith<$Res> {
  factory $AdapterReply_TradesCopyWith(AdapterReply_Trades value, $Res Function(AdapterReply_Trades) _then) = _$AdapterReply_TradesCopyWithImpl;
@useResult
$Res call({
 List<WireTrade> field0
});




}
/// @nodoc
class _$AdapterReply_TradesCopyWithImpl<$Res>
    implements $AdapterReply_TradesCopyWith<$Res> {
  _$AdapterReply_TradesCopyWithImpl(this._self, this._then);

  final AdapterReply_Trades _self;
  final $Res Function(AdapterReply_Trades) _then;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AdapterReply_Trades(
null == field0 ? _self._field0 : field0 // ignore: cast_nullable_to_non_nullable
as List<WireTrade>,
  ));
}


}

/// @nodoc


class AdapterReply_OrderBook extends AdapterReply {
  const AdapterReply_OrderBook(this.field0): super._();


 final  WireOrderBook field0;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterReply_OrderBookCopyWith<AdapterReply_OrderBook> get copyWith => _$AdapterReply_OrderBookCopyWithImpl<AdapterReply_OrderBook>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterReply_OrderBook&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'AdapterReply.orderBook(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AdapterReply_OrderBookCopyWith<$Res> implements $AdapterReplyCopyWith<$Res> {
  factory $AdapterReply_OrderBookCopyWith(AdapterReply_OrderBook value, $Res Function(AdapterReply_OrderBook) _then) = _$AdapterReply_OrderBookCopyWithImpl;
@useResult
$Res call({
 WireOrderBook field0
});




}
/// @nodoc
class _$AdapterReply_OrderBookCopyWithImpl<$Res>
    implements $AdapterReply_OrderBookCopyWith<$Res> {
  _$AdapterReply_OrderBookCopyWithImpl(this._self, this._then);

  final AdapterReply_OrderBook _self;
  final $Res Function(AdapterReply_OrderBook) _then;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AdapterReply_OrderBook(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireOrderBook,
  ));
}


}

/// @nodoc


class AdapterReply_Ticker extends AdapterReply {
  const AdapterReply_Ticker(this.field0): super._();


 final  WireTicker field0;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterReply_TickerCopyWith<AdapterReply_Ticker> get copyWith => _$AdapterReply_TickerCopyWithImpl<AdapterReply_Ticker>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterReply_Ticker&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'AdapterReply.ticker(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AdapterReply_TickerCopyWith<$Res> implements $AdapterReplyCopyWith<$Res> {
  factory $AdapterReply_TickerCopyWith(AdapterReply_Ticker value, $Res Function(AdapterReply_Ticker) _then) = _$AdapterReply_TickerCopyWithImpl;
@useResult
$Res call({
 WireTicker field0
});




}
/// @nodoc
class _$AdapterReply_TickerCopyWithImpl<$Res>
    implements $AdapterReply_TickerCopyWith<$Res> {
  _$AdapterReply_TickerCopyWithImpl(this._self, this._then);

  final AdapterReply_Ticker _self;
  final $Res Function(AdapterReply_Ticker) _then;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AdapterReply_Ticker(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireTicker,
  ));
}


}

/// @nodoc


class AdapterReply_Candles extends AdapterReply {
  const AdapterReply_Candles(final  List<WireCandle> field0): _field0 = field0,super._();


 final  List<WireCandle> _field0;
 List<WireCandle> get field0 {
  if (_field0 is EqualUnmodifiableListView) return _field0;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_field0);
}


/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterReply_CandlesCopyWith<AdapterReply_Candles> get copyWith => _$AdapterReply_CandlesCopyWithImpl<AdapterReply_Candles>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterReply_Candles&&const DeepCollectionEquality().equals(other._field0, _field0));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_field0));

@override
String toString() {
  return 'AdapterReply.candles(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AdapterReply_CandlesCopyWith<$Res> implements $AdapterReplyCopyWith<$Res> {
  factory $AdapterReply_CandlesCopyWith(AdapterReply_Candles value, $Res Function(AdapterReply_Candles) _then) = _$AdapterReply_CandlesCopyWithImpl;
@useResult
$Res call({
 List<WireCandle> field0
});




}
/// @nodoc
class _$AdapterReply_CandlesCopyWithImpl<$Res>
    implements $AdapterReply_CandlesCopyWith<$Res> {
  _$AdapterReply_CandlesCopyWithImpl(this._self, this._then);

  final AdapterReply_Candles _self;
  final $Res Function(AdapterReply_Candles) _then;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AdapterReply_Candles(
null == field0 ? _self._field0 : field0 // ignore: cast_nullable_to_non_nullable
as List<WireCandle>,
  ));
}


}

/// @nodoc


class AdapterReply_Balances extends AdapterReply {
  const AdapterReply_Balances(final  List<WireBalance> field0): _field0 = field0,super._();


 final  List<WireBalance> _field0;
 List<WireBalance> get field0 {
  if (_field0 is EqualUnmodifiableListView) return _field0;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_field0);
}


/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterReply_BalancesCopyWith<AdapterReply_Balances> get copyWith => _$AdapterReply_BalancesCopyWithImpl<AdapterReply_Balances>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterReply_Balances&&const DeepCollectionEquality().equals(other._field0, _field0));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_field0));

@override
String toString() {
  return 'AdapterReply.balances(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AdapterReply_BalancesCopyWith<$Res> implements $AdapterReplyCopyWith<$Res> {
  factory $AdapterReply_BalancesCopyWith(AdapterReply_Balances value, $Res Function(AdapterReply_Balances) _then) = _$AdapterReply_BalancesCopyWithImpl;
@useResult
$Res call({
 List<WireBalance> field0
});




}
/// @nodoc
class _$AdapterReply_BalancesCopyWithImpl<$Res>
    implements $AdapterReply_BalancesCopyWith<$Res> {
  _$AdapterReply_BalancesCopyWithImpl(this._self, this._then);

  final AdapterReply_Balances _self;
  final $Res Function(AdapterReply_Balances) _then;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AdapterReply_Balances(
null == field0 ? _self._field0 : field0 // ignore: cast_nullable_to_non_nullable
as List<WireBalance>,
  ));
}


}

/// @nodoc


class AdapterReply_OrderRules extends AdapterReply {
  const AdapterReply_OrderRules(this.field0): super._();


 final  WireOrderRules field0;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterReply_OrderRulesCopyWith<AdapterReply_OrderRules> get copyWith => _$AdapterReply_OrderRulesCopyWithImpl<AdapterReply_OrderRules>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterReply_OrderRules&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'AdapterReply.orderRules(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AdapterReply_OrderRulesCopyWith<$Res> implements $AdapterReplyCopyWith<$Res> {
  factory $AdapterReply_OrderRulesCopyWith(AdapterReply_OrderRules value, $Res Function(AdapterReply_OrderRules) _then) = _$AdapterReply_OrderRulesCopyWithImpl;
@useResult
$Res call({
 WireOrderRules field0
});




}
/// @nodoc
class _$AdapterReply_OrderRulesCopyWithImpl<$Res>
    implements $AdapterReply_OrderRulesCopyWith<$Res> {
  _$AdapterReply_OrderRulesCopyWithImpl(this._self, this._then);

  final AdapterReply_OrderRules _self;
  final $Res Function(AdapterReply_OrderRules) _then;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AdapterReply_OrderRules(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireOrderRules,
  ));
}


}

/// @nodoc


class AdapterReply_AssetNetworks extends AdapterReply {
  const AdapterReply_AssetNetworks(final  List<WireAssetNetwork> field0): _field0 = field0,super._();


 final  List<WireAssetNetwork> _field0;
 List<WireAssetNetwork> get field0 {
  if (_field0 is EqualUnmodifiableListView) return _field0;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_field0);
}


/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterReply_AssetNetworksCopyWith<AdapterReply_AssetNetworks> get copyWith => _$AdapterReply_AssetNetworksCopyWithImpl<AdapterReply_AssetNetworks>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterReply_AssetNetworks&&const DeepCollectionEquality().equals(other._field0, _field0));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_field0));

@override
String toString() {
  return 'AdapterReply.assetNetworks(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AdapterReply_AssetNetworksCopyWith<$Res> implements $AdapterReplyCopyWith<$Res> {
  factory $AdapterReply_AssetNetworksCopyWith(AdapterReply_AssetNetworks value, $Res Function(AdapterReply_AssetNetworks) _then) = _$AdapterReply_AssetNetworksCopyWithImpl;
@useResult
$Res call({
 List<WireAssetNetwork> field0
});




}
/// @nodoc
class _$AdapterReply_AssetNetworksCopyWithImpl<$Res>
    implements $AdapterReply_AssetNetworksCopyWith<$Res> {
  _$AdapterReply_AssetNetworksCopyWithImpl(this._self, this._then);

  final AdapterReply_AssetNetworks _self;
  final $Res Function(AdapterReply_AssetNetworks) _then;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AdapterReply_AssetNetworks(
null == field0 ? _self._field0 : field0 // ignore: cast_nullable_to_non_nullable
as List<WireAssetNetwork>,
  ));
}


}

/// @nodoc


class AdapterReply_DepositAddress extends AdapterReply {
  const AdapterReply_DepositAddress(this.field0): super._();


 final  WireDepositAddress field0;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterReply_DepositAddressCopyWith<AdapterReply_DepositAddress> get copyWith => _$AdapterReply_DepositAddressCopyWithImpl<AdapterReply_DepositAddress>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterReply_DepositAddress&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'AdapterReply.depositAddress(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AdapterReply_DepositAddressCopyWith<$Res> implements $AdapterReplyCopyWith<$Res> {
  factory $AdapterReply_DepositAddressCopyWith(AdapterReply_DepositAddress value, $Res Function(AdapterReply_DepositAddress) _then) = _$AdapterReply_DepositAddressCopyWithImpl;
@useResult
$Res call({
 WireDepositAddress field0
});




}
/// @nodoc
class _$AdapterReply_DepositAddressCopyWithImpl<$Res>
    implements $AdapterReply_DepositAddressCopyWith<$Res> {
  _$AdapterReply_DepositAddressCopyWithImpl(this._self, this._then);

  final AdapterReply_DepositAddress _self;
  final $Res Function(AdapterReply_DepositAddress) _then;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AdapterReply_DepositAddress(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireDepositAddress,
  ));
}


}

/// @nodoc


class AdapterReply_CreateDepositAddress extends AdapterReply {
  const AdapterReply_CreateDepositAddress(this.field0): super._();


 final  WireDepositAddress field0;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterReply_CreateDepositAddressCopyWith<AdapterReply_CreateDepositAddress> get copyWith => _$AdapterReply_CreateDepositAddressCopyWithImpl<AdapterReply_CreateDepositAddress>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterReply_CreateDepositAddress&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'AdapterReply.createDepositAddress(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AdapterReply_CreateDepositAddressCopyWith<$Res> implements $AdapterReplyCopyWith<$Res> {
  factory $AdapterReply_CreateDepositAddressCopyWith(AdapterReply_CreateDepositAddress value, $Res Function(AdapterReply_CreateDepositAddress) _then) = _$AdapterReply_CreateDepositAddressCopyWithImpl;
@useResult
$Res call({
 WireDepositAddress field0
});




}
/// @nodoc
class _$AdapterReply_CreateDepositAddressCopyWithImpl<$Res>
    implements $AdapterReply_CreateDepositAddressCopyWith<$Res> {
  _$AdapterReply_CreateDepositAddressCopyWithImpl(this._self, this._then);

  final AdapterReply_CreateDepositAddress _self;
  final $Res Function(AdapterReply_CreateDepositAddress) _then;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AdapterReply_CreateDepositAddress(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireDepositAddress,
  ));
}


}

/// @nodoc


class AdapterReply_PrepareWithdrawal extends AdapterReply {
  const AdapterReply_PrepareWithdrawal(this.field0): super._();


 final  WireWithdrawalQuote field0;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterReply_PrepareWithdrawalCopyWith<AdapterReply_PrepareWithdrawal> get copyWith => _$AdapterReply_PrepareWithdrawalCopyWithImpl<AdapterReply_PrepareWithdrawal>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterReply_PrepareWithdrawal&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'AdapterReply.prepareWithdrawal(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AdapterReply_PrepareWithdrawalCopyWith<$Res> implements $AdapterReplyCopyWith<$Res> {
  factory $AdapterReply_PrepareWithdrawalCopyWith(AdapterReply_PrepareWithdrawal value, $Res Function(AdapterReply_PrepareWithdrawal) _then) = _$AdapterReply_PrepareWithdrawalCopyWithImpl;
@useResult
$Res call({
 WireWithdrawalQuote field0
});




}
/// @nodoc
class _$AdapterReply_PrepareWithdrawalCopyWithImpl<$Res>
    implements $AdapterReply_PrepareWithdrawalCopyWith<$Res> {
  _$AdapterReply_PrepareWithdrawalCopyWithImpl(this._self, this._then);

  final AdapterReply_PrepareWithdrawal _self;
  final $Res Function(AdapterReply_PrepareWithdrawal) _then;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AdapterReply_PrepareWithdrawal(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireWithdrawalQuote,
  ));
}


}

/// @nodoc


class AdapterReply_Withdraw extends AdapterReply {
  const AdapterReply_Withdraw(this.field0): super._();


 final  WireWithdrawal field0;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterReply_WithdrawCopyWith<AdapterReply_Withdraw> get copyWith => _$AdapterReply_WithdrawCopyWithImpl<AdapterReply_Withdraw>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterReply_Withdraw&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'AdapterReply.withdraw(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AdapterReply_WithdrawCopyWith<$Res> implements $AdapterReplyCopyWith<$Res> {
  factory $AdapterReply_WithdrawCopyWith(AdapterReply_Withdraw value, $Res Function(AdapterReply_Withdraw) _then) = _$AdapterReply_WithdrawCopyWithImpl;
@useResult
$Res call({
 WireWithdrawal field0
});




}
/// @nodoc
class _$AdapterReply_WithdrawCopyWithImpl<$Res>
    implements $AdapterReply_WithdrawCopyWith<$Res> {
  _$AdapterReply_WithdrawCopyWithImpl(this._self, this._then);

  final AdapterReply_Withdraw _self;
  final $Res Function(AdapterReply_Withdraw) _then;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AdapterReply_Withdraw(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireWithdrawal,
  ));
}


}

/// @nodoc


class AdapterReply_Deposits extends AdapterReply {
  const AdapterReply_Deposits(this.field0): super._();


 final  WireDepositPage field0;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterReply_DepositsCopyWith<AdapterReply_Deposits> get copyWith => _$AdapterReply_DepositsCopyWithImpl<AdapterReply_Deposits>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterReply_Deposits&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'AdapterReply.deposits(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AdapterReply_DepositsCopyWith<$Res> implements $AdapterReplyCopyWith<$Res> {
  factory $AdapterReply_DepositsCopyWith(AdapterReply_Deposits value, $Res Function(AdapterReply_Deposits) _then) = _$AdapterReply_DepositsCopyWithImpl;
@useResult
$Res call({
 WireDepositPage field0
});




}
/// @nodoc
class _$AdapterReply_DepositsCopyWithImpl<$Res>
    implements $AdapterReply_DepositsCopyWith<$Res> {
  _$AdapterReply_DepositsCopyWithImpl(this._self, this._then);

  final AdapterReply_Deposits _self;
  final $Res Function(AdapterReply_Deposits) _then;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AdapterReply_Deposits(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireDepositPage,
  ));
}


}

/// @nodoc


class AdapterReply_Withdrawals extends AdapterReply {
  const AdapterReply_Withdrawals(this.field0): super._();


 final  WireWithdrawalPage field0;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterReply_WithdrawalsCopyWith<AdapterReply_Withdrawals> get copyWith => _$AdapterReply_WithdrawalsCopyWithImpl<AdapterReply_Withdrawals>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterReply_Withdrawals&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'AdapterReply.withdrawals(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AdapterReply_WithdrawalsCopyWith<$Res> implements $AdapterReplyCopyWith<$Res> {
  factory $AdapterReply_WithdrawalsCopyWith(AdapterReply_Withdrawals value, $Res Function(AdapterReply_Withdrawals) _then) = _$AdapterReply_WithdrawalsCopyWithImpl;
@useResult
$Res call({
 WireWithdrawalPage field0
});




}
/// @nodoc
class _$AdapterReply_WithdrawalsCopyWithImpl<$Res>
    implements $AdapterReply_WithdrawalsCopyWith<$Res> {
  _$AdapterReply_WithdrawalsCopyWithImpl(this._self, this._then);

  final AdapterReply_Withdrawals _self;
  final $Res Function(AdapterReply_Withdrawals) _then;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AdapterReply_Withdrawals(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireWithdrawalPage,
  ));
}


}

/// @nodoc


class AdapterReply_OpenOrders extends AdapterReply {
  const AdapterReply_OpenOrders(final  List<WireOrder> field0): _field0 = field0,super._();


 final  List<WireOrder> _field0;
 List<WireOrder> get field0 {
  if (_field0 is EqualUnmodifiableListView) return _field0;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_field0);
}


/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterReply_OpenOrdersCopyWith<AdapterReply_OpenOrders> get copyWith => _$AdapterReply_OpenOrdersCopyWithImpl<AdapterReply_OpenOrders>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterReply_OpenOrders&&const DeepCollectionEquality().equals(other._field0, _field0));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_field0));

@override
String toString() {
  return 'AdapterReply.openOrders(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AdapterReply_OpenOrdersCopyWith<$Res> implements $AdapterReplyCopyWith<$Res> {
  factory $AdapterReply_OpenOrdersCopyWith(AdapterReply_OpenOrders value, $Res Function(AdapterReply_OpenOrders) _then) = _$AdapterReply_OpenOrdersCopyWithImpl;
@useResult
$Res call({
 List<WireOrder> field0
});




}
/// @nodoc
class _$AdapterReply_OpenOrdersCopyWithImpl<$Res>
    implements $AdapterReply_OpenOrdersCopyWith<$Res> {
  _$AdapterReply_OpenOrdersCopyWithImpl(this._self, this._then);

  final AdapterReply_OpenOrders _self;
  final $Res Function(AdapterReply_OpenOrders) _then;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AdapterReply_OpenOrders(
null == field0 ? _self._field0 : field0 // ignore: cast_nullable_to_non_nullable
as List<WireOrder>,
  ));
}


}

/// @nodoc


class AdapterReply_Order extends AdapterReply {
  const AdapterReply_Order(this.field0): super._();


 final  WireOrder field0;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterReply_OrderCopyWith<AdapterReply_Order> get copyWith => _$AdapterReply_OrderCopyWithImpl<AdapterReply_Order>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterReply_Order&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'AdapterReply.order(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AdapterReply_OrderCopyWith<$Res> implements $AdapterReplyCopyWith<$Res> {
  factory $AdapterReply_OrderCopyWith(AdapterReply_Order value, $Res Function(AdapterReply_Order) _then) = _$AdapterReply_OrderCopyWithImpl;
@useResult
$Res call({
 WireOrder field0
});




}
/// @nodoc
class _$AdapterReply_OrderCopyWithImpl<$Res>
    implements $AdapterReply_OrderCopyWith<$Res> {
  _$AdapterReply_OrderCopyWithImpl(this._self, this._then);

  final AdapterReply_Order _self;
  final $Res Function(AdapterReply_Order) _then;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AdapterReply_Order(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireOrder,
  ));
}


}

/// @nodoc


class AdapterReply_OrdersByIds extends AdapterReply {
  const AdapterReply_OrdersByIds(final  List<WireOrder> field0): _field0 = field0,super._();


 final  List<WireOrder> _field0;
 List<WireOrder> get field0 {
  if (_field0 is EqualUnmodifiableListView) return _field0;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_field0);
}


/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterReply_OrdersByIdsCopyWith<AdapterReply_OrdersByIds> get copyWith => _$AdapterReply_OrdersByIdsCopyWithImpl<AdapterReply_OrdersByIds>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterReply_OrdersByIds&&const DeepCollectionEquality().equals(other._field0, _field0));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_field0));

@override
String toString() {
  return 'AdapterReply.ordersByIds(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AdapterReply_OrdersByIdsCopyWith<$Res> implements $AdapterReplyCopyWith<$Res> {
  factory $AdapterReply_OrdersByIdsCopyWith(AdapterReply_OrdersByIds value, $Res Function(AdapterReply_OrdersByIds) _then) = _$AdapterReply_OrdersByIdsCopyWithImpl;
@useResult
$Res call({
 List<WireOrder> field0
});




}
/// @nodoc
class _$AdapterReply_OrdersByIdsCopyWithImpl<$Res>
    implements $AdapterReply_OrdersByIdsCopyWith<$Res> {
  _$AdapterReply_OrdersByIdsCopyWithImpl(this._self, this._then);

  final AdapterReply_OrdersByIds _self;
  final $Res Function(AdapterReply_OrdersByIds) _then;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AdapterReply_OrdersByIds(
null == field0 ? _self._field0 : field0 // ignore: cast_nullable_to_non_nullable
as List<WireOrder>,
  ));
}


}

/// @nodoc


class AdapterReply_OrderHistory extends AdapterReply {
  const AdapterReply_OrderHistory(this.field0): super._();


 final  WireOrderPage field0;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterReply_OrderHistoryCopyWith<AdapterReply_OrderHistory> get copyWith => _$AdapterReply_OrderHistoryCopyWithImpl<AdapterReply_OrderHistory>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterReply_OrderHistory&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'AdapterReply.orderHistory(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AdapterReply_OrderHistoryCopyWith<$Res> implements $AdapterReplyCopyWith<$Res> {
  factory $AdapterReply_OrderHistoryCopyWith(AdapterReply_OrderHistory value, $Res Function(AdapterReply_OrderHistory) _then) = _$AdapterReply_OrderHistoryCopyWithImpl;
@useResult
$Res call({
 WireOrderPage field0
});




}
/// @nodoc
class _$AdapterReply_OrderHistoryCopyWithImpl<$Res>
    implements $AdapterReply_OrderHistoryCopyWith<$Res> {
  _$AdapterReply_OrderHistoryCopyWithImpl(this._self, this._then);

  final AdapterReply_OrderHistory _self;
  final $Res Function(AdapterReply_OrderHistory) _then;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AdapterReply_OrderHistory(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireOrderPage,
  ));
}


}

/// @nodoc


class AdapterReply_PlaceOrder extends AdapterReply {
  const AdapterReply_PlaceOrder(this.field0): super._();


 final  WireOrder field0;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterReply_PlaceOrderCopyWith<AdapterReply_PlaceOrder> get copyWith => _$AdapterReply_PlaceOrderCopyWithImpl<AdapterReply_PlaceOrder>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterReply_PlaceOrder&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'AdapterReply.placeOrder(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AdapterReply_PlaceOrderCopyWith<$Res> implements $AdapterReplyCopyWith<$Res> {
  factory $AdapterReply_PlaceOrderCopyWith(AdapterReply_PlaceOrder value, $Res Function(AdapterReply_PlaceOrder) _then) = _$AdapterReply_PlaceOrderCopyWithImpl;
@useResult
$Res call({
 WireOrder field0
});




}
/// @nodoc
class _$AdapterReply_PlaceOrderCopyWithImpl<$Res>
    implements $AdapterReply_PlaceOrderCopyWith<$Res> {
  _$AdapterReply_PlaceOrderCopyWithImpl(this._self, this._then);

  final AdapterReply_PlaceOrder _self;
  final $Res Function(AdapterReply_PlaceOrder) _then;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AdapterReply_PlaceOrder(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireOrder,
  ));
}


}

/// @nodoc


class AdapterReply_CancelOrders extends AdapterReply {
  const AdapterReply_CancelOrders(this.field0): super._();


 final  WireCancelOrdersResult field0;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterReply_CancelOrdersCopyWith<AdapterReply_CancelOrders> get copyWith => _$AdapterReply_CancelOrdersCopyWithImpl<AdapterReply_CancelOrders>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterReply_CancelOrders&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'AdapterReply.cancelOrders(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AdapterReply_CancelOrdersCopyWith<$Res> implements $AdapterReplyCopyWith<$Res> {
  factory $AdapterReply_CancelOrdersCopyWith(AdapterReply_CancelOrders value, $Res Function(AdapterReply_CancelOrders) _then) = _$AdapterReply_CancelOrdersCopyWithImpl;
@useResult
$Res call({
 WireCancelOrdersResult field0
});




}
/// @nodoc
class _$AdapterReply_CancelOrdersCopyWithImpl<$Res>
    implements $AdapterReply_CancelOrdersCopyWith<$Res> {
  _$AdapterReply_CancelOrdersCopyWithImpl(this._self, this._then);

  final AdapterReply_CancelOrders _self;
  final $Res Function(AdapterReply_CancelOrders) _then;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AdapterReply_CancelOrders(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireCancelOrdersResult,
  ));
}


}

/// @nodoc


class AdapterReply_Positions extends AdapterReply {
  const AdapterReply_Positions(final  List<WirePosition> field0): _field0 = field0,super._();


 final  List<WirePosition> _field0;
 List<WirePosition> get field0 {
  if (_field0 is EqualUnmodifiableListView) return _field0;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_field0);
}


/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterReply_PositionsCopyWith<AdapterReply_Positions> get copyWith => _$AdapterReply_PositionsCopyWithImpl<AdapterReply_Positions>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterReply_Positions&&const DeepCollectionEquality().equals(other._field0, _field0));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_field0));

@override
String toString() {
  return 'AdapterReply.positions(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AdapterReply_PositionsCopyWith<$Res> implements $AdapterReplyCopyWith<$Res> {
  factory $AdapterReply_PositionsCopyWith(AdapterReply_Positions value, $Res Function(AdapterReply_Positions) _then) = _$AdapterReply_PositionsCopyWithImpl;
@useResult
$Res call({
 List<WirePosition> field0
});




}
/// @nodoc
class _$AdapterReply_PositionsCopyWithImpl<$Res>
    implements $AdapterReply_PositionsCopyWith<$Res> {
  _$AdapterReply_PositionsCopyWithImpl(this._self, this._then);

  final AdapterReply_Positions _self;
  final $Res Function(AdapterReply_Positions) _then;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AdapterReply_Positions(
null == field0 ? _self._field0 : field0 // ignore: cast_nullable_to_non_nullable
as List<WirePosition>,
  ));
}


}

/// @nodoc


class AdapterReply_MarginSummary extends AdapterReply {
  const AdapterReply_MarginSummary(this.field0): super._();


 final  WireMarginSummary field0;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterReply_MarginSummaryCopyWith<AdapterReply_MarginSummary> get copyWith => _$AdapterReply_MarginSummaryCopyWithImpl<AdapterReply_MarginSummary>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterReply_MarginSummary&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'AdapterReply.marginSummary(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AdapterReply_MarginSummaryCopyWith<$Res> implements $AdapterReplyCopyWith<$Res> {
  factory $AdapterReply_MarginSummaryCopyWith(AdapterReply_MarginSummary value, $Res Function(AdapterReply_MarginSummary) _then) = _$AdapterReply_MarginSummaryCopyWithImpl;
@useResult
$Res call({
 WireMarginSummary field0
});




}
/// @nodoc
class _$AdapterReply_MarginSummaryCopyWithImpl<$Res>
    implements $AdapterReply_MarginSummaryCopyWith<$Res> {
  _$AdapterReply_MarginSummaryCopyWithImpl(this._self, this._then);

  final AdapterReply_MarginSummary _self;
  final $Res Function(AdapterReply_MarginSummary) _then;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AdapterReply_MarginSummary(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireMarginSummary,
  ));
}


}

/// @nodoc


class AdapterReply_FundingRates extends AdapterReply {
  const AdapterReply_FundingRates(this.field0): super._();


 final  WireFundingRatePage field0;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterReply_FundingRatesCopyWith<AdapterReply_FundingRates> get copyWith => _$AdapterReply_FundingRatesCopyWithImpl<AdapterReply_FundingRates>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterReply_FundingRates&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'AdapterReply.fundingRates(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AdapterReply_FundingRatesCopyWith<$Res> implements $AdapterReplyCopyWith<$Res> {
  factory $AdapterReply_FundingRatesCopyWith(AdapterReply_FundingRates value, $Res Function(AdapterReply_FundingRates) _then) = _$AdapterReply_FundingRatesCopyWithImpl;
@useResult
$Res call({
 WireFundingRatePage field0
});




}
/// @nodoc
class _$AdapterReply_FundingRatesCopyWithImpl<$Res>
    implements $AdapterReply_FundingRatesCopyWith<$Res> {
  _$AdapterReply_FundingRatesCopyWithImpl(this._self, this._then);

  final AdapterReply_FundingRates _self;
  final $Res Function(AdapterReply_FundingRates) _then;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AdapterReply_FundingRates(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireFundingRatePage,
  ));
}


}

/// @nodoc


class AdapterReply_FundingPayments extends AdapterReply {
  const AdapterReply_FundingPayments(this.field0): super._();


 final  WireFundingPaymentPage field0;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterReply_FundingPaymentsCopyWith<AdapterReply_FundingPayments> get copyWith => _$AdapterReply_FundingPaymentsCopyWithImpl<AdapterReply_FundingPayments>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterReply_FundingPayments&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'AdapterReply.fundingPayments(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AdapterReply_FundingPaymentsCopyWith<$Res> implements $AdapterReplyCopyWith<$Res> {
  factory $AdapterReply_FundingPaymentsCopyWith(AdapterReply_FundingPayments value, $Res Function(AdapterReply_FundingPayments) _then) = _$AdapterReply_FundingPaymentsCopyWithImpl;
@useResult
$Res call({
 WireFundingPaymentPage field0
});




}
/// @nodoc
class _$AdapterReply_FundingPaymentsCopyWithImpl<$Res>
    implements $AdapterReply_FundingPaymentsCopyWith<$Res> {
  _$AdapterReply_FundingPaymentsCopyWithImpl(this._self, this._then);

  final AdapterReply_FundingPayments _self;
  final $Res Function(AdapterReply_FundingPayments) _then;

/// Create a copy of AdapterReply
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AdapterReply_FundingPayments(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireFundingPaymentPage,
  ));
}


}

/// @nodoc


class AdapterReply_Unit extends AdapterReply {
  const AdapterReply_Unit(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterReply_Unit);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AdapterReply.unit()';
}


}




/// @nodoc
mixin _$AdapterResult {

 Object get field0;



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterResult&&const DeepCollectionEquality().equals(other.field0, field0));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(field0));

@override
String toString() {
  return 'AdapterResult(field0: $field0)';
}


}

/// @nodoc
class $AdapterResultCopyWith<$Res>  {
$AdapterResultCopyWith(AdapterResult _, $Res Function(AdapterResult) __);
}


/// Adds pattern-matching-related methods to [AdapterResult].
extension AdapterResultPatterns on AdapterResult {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( AdapterResult_Success value)?  success,TResult Function( AdapterResult_Error value)?  error,required TResult orElse(),}){
final _that = this;
switch (_that) {
case AdapterResult_Success() when success != null:
return success(_that);case AdapterResult_Error() when error != null:
return error(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( AdapterResult_Success value)  success,required TResult Function( AdapterResult_Error value)  error,}){
final _that = this;
switch (_that) {
case AdapterResult_Success():
return success(_that);case AdapterResult_Error():
return error(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( AdapterResult_Success value)?  success,TResult? Function( AdapterResult_Error value)?  error,}){
final _that = this;
switch (_that) {
case AdapterResult_Success() when success != null:
return success(_that);case AdapterResult_Error() when error != null:
return error(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( AdapterReply field0)?  success,TResult Function( NativeError field0)?  error,required TResult orElse(),}) {final _that = this;
switch (_that) {
case AdapterResult_Success() when success != null:
return success(_that.field0);case AdapterResult_Error() when error != null:
return error(_that.field0);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( AdapterReply field0)  success,required TResult Function( NativeError field0)  error,}) {final _that = this;
switch (_that) {
case AdapterResult_Success():
return success(_that.field0);case AdapterResult_Error():
return error(_that.field0);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( AdapterReply field0)?  success,TResult? Function( NativeError field0)?  error,}) {final _that = this;
switch (_that) {
case AdapterResult_Success() when success != null:
return success(_that.field0);case AdapterResult_Error() when error != null:
return error(_that.field0);case _:
  return null;

}
}

}

/// @nodoc


class AdapterResult_Success extends AdapterResult {
  const AdapterResult_Success(this.field0): super._();


@override final  AdapterReply field0;

/// Create a copy of AdapterResult
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterResult_SuccessCopyWith<AdapterResult_Success> get copyWith => _$AdapterResult_SuccessCopyWithImpl<AdapterResult_Success>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterResult_Success&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'AdapterResult.success(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AdapterResult_SuccessCopyWith<$Res> implements $AdapterResultCopyWith<$Res> {
  factory $AdapterResult_SuccessCopyWith(AdapterResult_Success value, $Res Function(AdapterResult_Success) _then) = _$AdapterResult_SuccessCopyWithImpl;
@useResult
$Res call({
 AdapterReply field0
});


$AdapterReplyCopyWith<$Res> get field0;

}
/// @nodoc
class _$AdapterResult_SuccessCopyWithImpl<$Res>
    implements $AdapterResult_SuccessCopyWith<$Res> {
  _$AdapterResult_SuccessCopyWithImpl(this._self, this._then);

  final AdapterResult_Success _self;
  final $Res Function(AdapterResult_Success) _then;

/// Create a copy of AdapterResult
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AdapterResult_Success(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as AdapterReply,
  ));
}

/// Create a copy of AdapterResult
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AdapterReplyCopyWith<$Res> get field0 {

  return $AdapterReplyCopyWith<$Res>(_self.field0, (value) {
    return _then(_self.copyWith(field0: value));
  });
}
}

/// @nodoc


class AdapterResult_Error extends AdapterResult {
  const AdapterResult_Error(this.field0): super._();


@override final  NativeError field0;

/// Create a copy of AdapterResult
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AdapterResult_ErrorCopyWith<AdapterResult_Error> get copyWith => _$AdapterResult_ErrorCopyWithImpl<AdapterResult_Error>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AdapterResult_Error&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'AdapterResult.error(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AdapterResult_ErrorCopyWith<$Res> implements $AdapterResultCopyWith<$Res> {
  factory $AdapterResult_ErrorCopyWith(AdapterResult_Error value, $Res Function(AdapterResult_Error) _then) = _$AdapterResult_ErrorCopyWithImpl;
@useResult
$Res call({
 NativeError field0
});




}
/// @nodoc
class _$AdapterResult_ErrorCopyWithImpl<$Res>
    implements $AdapterResult_ErrorCopyWith<$Res> {
  _$AdapterResult_ErrorCopyWithImpl(this._self, this._then);

  final AdapterResult_Error _self;
  final $Res Function(AdapterResult_Error) _then;

/// Create a copy of AdapterResult
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AdapterResult_Error(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as NativeError,
  ));
}


}

/// @nodoc
mixin _$WireFeed {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireFeed);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireFeed()';
}


}

/// @nodoc
class $WireFeedCopyWith<$Res>  {
$WireFeedCopyWith(WireFeed _, $Res Function(WireFeed) __);
}


/// Adds pattern-matching-related methods to [WireFeed].
extension WireFeedPatterns on WireFeed {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WireFeed_Trades value)?  trades,TResult Function( WireFeed_OrderBook value)?  orderBook,TResult Function( WireFeed_Ticker value)?  ticker,TResult Function( WireFeed_Candles value)?  candles,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WireFeed_Trades() when trades != null:
return trades(_that);case WireFeed_OrderBook() when orderBook != null:
return orderBook(_that);case WireFeed_Ticker() when ticker != null:
return ticker(_that);case WireFeed_Candles() when candles != null:
return candles(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WireFeed_Trades value)  trades,required TResult Function( WireFeed_OrderBook value)  orderBook,required TResult Function( WireFeed_Ticker value)  ticker,required TResult Function( WireFeed_Candles value)  candles,}){
final _that = this;
switch (_that) {
case WireFeed_Trades():
return trades(_that);case WireFeed_OrderBook():
return orderBook(_that);case WireFeed_Ticker():
return ticker(_that);case WireFeed_Candles():
return candles(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WireFeed_Trades value)?  trades,TResult? Function( WireFeed_OrderBook value)?  orderBook,TResult? Function( WireFeed_Ticker value)?  ticker,TResult? Function( WireFeed_Candles value)?  candles,}){
final _that = this;
switch (_that) {
case WireFeed_Trades() when trades != null:
return trades(_that);case WireFeed_OrderBook() when orderBook != null:
return orderBook(_that);case WireFeed_Ticker() when ticker != null:
return ticker(_that);case WireFeed_Candles() when candles != null:
return candles(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  trades,TResult Function()?  orderBook,TResult Function()?  ticker,TResult Function( WireInterval field0)?  candles,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WireFeed_Trades() when trades != null:
return trades();case WireFeed_OrderBook() when orderBook != null:
return orderBook();case WireFeed_Ticker() when ticker != null:
return ticker();case WireFeed_Candles() when candles != null:
return candles(_that.field0);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  trades,required TResult Function()  orderBook,required TResult Function()  ticker,required TResult Function( WireInterval field0)  candles,}) {final _that = this;
switch (_that) {
case WireFeed_Trades():
return trades();case WireFeed_OrderBook():
return orderBook();case WireFeed_Ticker():
return ticker();case WireFeed_Candles():
return candles(_that.field0);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  trades,TResult? Function()?  orderBook,TResult? Function()?  ticker,TResult? Function( WireInterval field0)?  candles,}) {final _that = this;
switch (_that) {
case WireFeed_Trades() when trades != null:
return trades();case WireFeed_OrderBook() when orderBook != null:
return orderBook();case WireFeed_Ticker() when ticker != null:
return ticker();case WireFeed_Candles() when candles != null:
return candles(_that.field0);case _:
  return null;

}
}

}

/// @nodoc


class WireFeed_Trades extends WireFeed {
  const WireFeed_Trades(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireFeed_Trades);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireFeed.trades()';
}


}




/// @nodoc


class WireFeed_OrderBook extends WireFeed {
  const WireFeed_OrderBook(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireFeed_OrderBook);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireFeed.orderBook()';
}


}




/// @nodoc


class WireFeed_Ticker extends WireFeed {
  const WireFeed_Ticker(): super._();







@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireFeed_Ticker);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WireFeed.ticker()';
}


}




/// @nodoc


class WireFeed_Candles extends WireFeed {
  const WireFeed_Candles(this.field0): super._();


 final  WireInterval field0;

/// Create a copy of WireFeed
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WireFeed_CandlesCopyWith<WireFeed_Candles> get copyWith => _$WireFeed_CandlesCopyWithImpl<WireFeed_Candles>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WireFeed_Candles&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'WireFeed.candles(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $WireFeed_CandlesCopyWith<$Res> implements $WireFeedCopyWith<$Res> {
  factory $WireFeed_CandlesCopyWith(WireFeed_Candles value, $Res Function(WireFeed_Candles) _then) = _$WireFeed_CandlesCopyWithImpl;
@useResult
$Res call({
 WireInterval field0
});




}
/// @nodoc
class _$WireFeed_CandlesCopyWithImpl<$Res>
    implements $WireFeed_CandlesCopyWith<$Res> {
  _$WireFeed_CandlesCopyWithImpl(this._self, this._then);

  final WireFeed_Candles _self;
  final $Res Function(WireFeed_Candles) _then;

/// Create a copy of WireFeed
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(WireFeed_Candles(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as WireInterval,
  ));
}


}

// dart format on

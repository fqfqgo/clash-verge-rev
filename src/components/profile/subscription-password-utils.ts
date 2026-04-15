const SUBSCRIPTION_NEED_PASSWORD = "SUBSCRIPTION_NEED_PASSWORD";
const SUBSCRIPTION_WRONG_PASSWORD = "SUBSCRIPTION_WRONG_PASSWORD";

export function isSubscriptionNeedPassword(err: unknown): boolean {
  const msg = typeof err === "string" ? err : ((err as Error)?.message ?? "");
  return msg.includes(SUBSCRIPTION_NEED_PASSWORD);
}

export function isSubscriptionWrongPassword(err: unknown): boolean {
  const msg = typeof err === "string" ? err : ((err as Error)?.message ?? "");
  return msg.includes(SUBSCRIPTION_WRONG_PASSWORD);
}

export function isSubscriptionPasswordError(err: unknown): boolean {
  return isSubscriptionNeedPassword(err) || isSubscriptionWrongPassword(err);
}

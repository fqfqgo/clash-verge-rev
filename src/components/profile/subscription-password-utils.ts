import { errorDetail } from '@/services/notice-service'

const SUBSCRIPTION_NEED_PASSWORD = 'SUBSCRIPTION_NEED_PASSWORD'
const SUBSCRIPTION_WRONG_PASSWORD = 'SUBSCRIPTION_WRONG_PASSWORD'

function isSubscriptionNeedPassword(err: unknown): boolean {
  return errorDetail(err).includes(SUBSCRIPTION_NEED_PASSWORD)
}

export function isSubscriptionWrongPassword(err: unknown): boolean {
  return errorDetail(err).includes(SUBSCRIPTION_WRONG_PASSWORD)
}

export function isSubscriptionPasswordError(err: unknown): boolean {
  return isSubscriptionNeedPassword(err) || isSubscriptionWrongPassword(err)
}

import type { MajorCurrencyAmount } from './Currency';

export interface DelegationWithEverything {
  owner: string;
  node_identity: string;
  amount: MajorCurrencyAmount;
  total_delegation: MajorCurrencyAmount | null;
  pledge_amount: MajorCurrencyAmount | null;
  block_height: bigint;
  delegated_on_iso_datetime: string;
  profit_margin_percent: number | null;
  avg_uptime_percent: number | null;
  stake_saturation: number | null;
  proxy: string | null;
  accumulated_rewards: MajorCurrencyAmount | null;
}

import { DelegationWithEverything, DelegationsSummaryResponse } from '@nymproject/types';
import { invokeWrapper } from './wrapper';

export const getMixNodeDelegationsForCurrentAccount = async () =>
  invokeWrapper<DelegationWithEverything[]>('get_all_mix_delegations');

export const getDelegationSummary = async () => invokeWrapper<DelegationsSummaryResponse>('get_delegation_summary');

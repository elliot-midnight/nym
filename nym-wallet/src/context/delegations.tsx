import React, { createContext, FC, useCallback, useContext, useEffect, useMemo, useState } from 'react';
<<<<<<< HEAD
import { getDelegationSummary } from 'src/requests/delegation';
=======
import { getDelegationSummary, getMixNodeDelegationsForCurrentAccount } from 'src/requests/delegation';
>>>>>>> 10ad612d (set total delegations)
import type { Network } from 'src/types';
import { DelegationWithEverything } from '@nymproject/types';

export type TDelegationContext = {
  isLoading: boolean;
  error?: string;
  delegations?: DelegationWithEverything[];
  totalDelegations?: string;
  refresh: () => Promise<void>;
  addDelegation: (newDelegation: DelegationWithEverything) => Promise<TDelegationTransaction>;
  updateDelegation: (newDelegation: DelegationWithEverything) => Promise<TDelegationTransaction>;
  undelegate: (mixnodeAddress: string) => Promise<TDelegationTransaction>;
};

export type TDelegationTransaction = {
  transactionUrl: string;
};

export const DelegationContext = createContext<TDelegationContext>({
  isLoading: true,
  refresh: async () => undefined,
  addDelegation: async () => {
    throw new Error('Not implemented');
  },
  updateDelegation: async () => {
    throw new Error('Not implemented');
  },
  undelegate: async () => {
    throw new Error('Not implemented');
  },
});

export const DelegationContextProvider: FC<{
  network?: Network;
}> = ({ network, children }) => {
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string>();
  const [delegations, setDelegations] = useState<undefined | DelegationWithEverything[]>();
  const [totalDelegations, setTotalDelegations] = useState<undefined | string>();

  const addDelegation = async (): Promise<TDelegationTransaction> => {
    throw new Error('Not implemented');
  };
  const updateDelegation = async (): Promise<TDelegationTransaction> => {
    throw new Error('Not implemented');
  };
  const undelegate = async (): Promise<TDelegationTransaction> => {
    throw new Error('Not implemented');
  };

  const resetState = () => {
    setIsLoading(true);
    setError(undefined);
    setTotalDelegations(undefined);
    setDelegations([]);
  };

  const refresh = useCallback(async () => {
    console.log('called');
    try {
      const data = await getDelegationSummary();
      setDelegations(data.delegations);
      setTotalDelegations(`${data.total_delegations.amount} ${data.total_delegations.denom}`);
    } catch (e) {
      setError((e as Error).message);
    }
    setIsLoading(false);
  }, [network]);

  useEffect(() => {
    // reset state and refresh
    resetState();
    refresh();
  }, [network]);

  const memoizedValue = useMemo(
    () => ({
      isLoading,
      error,
      delegations,
      totalDelegations,
      refresh,
      addDelegation,
      updateDelegation,
      undelegate,
    }),
    [isLoading, error, delegations, totalDelegations],
  );

  return <DelegationContext.Provider value={memoizedValue}>{children}</DelegationContext.Provider>;
};

export const useDelegationContext = () => useContext<TDelegationContext>(DelegationContext);

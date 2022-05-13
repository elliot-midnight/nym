import React, { createContext, FC, useCallback, useContext, useEffect, useMemo, useState } from 'react';
import { Network } from '../types';

type TRewardsContext = {
  isLoading: boolean;
  error?: string;
  totalRewards?: string;
  refresh: () => Promise<void>;
  redeemRewards: (mixnodeAddress: string) => Promise<TRewardsTransaction>;
  redeemAllRewards: () => Promise<TRewardsTransaction>;
};

export type TRewardsTransaction = {
  transactionUrl: string;
};

export const RewardsContext = createContext<TRewardsContext>({
  isLoading: true,
  refresh: async () => undefined,
  redeemRewards: async () => {
    throw new Error('Not implemented');
  },
  redeemAllRewards: async () => {
    throw new Error('Not implemented');
  },
});

export const RewardsContextProvider: FC<{
  network: Network;
}> = ({ network, children }) => {
  const [currentNetwork, setCurrentNetwork] = useState<undefined | Network>();
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string>();
  const [totalRewards, setTotalRewards] = useState<undefined | string>();

  const resetState = () => {
    setIsLoading(true);
    setError(undefined);
    setTotalRewards(undefined);
  };

  // TODO: implement
  const refresh = useCallback(async () => {
    resetState();
    // TODO: do work

    setIsLoading(false);
  }, [network]);

  useEffect(() => {
    if (currentNetwork !== network) {
      // reset state and refresh
      resetState();
      setCurrentNetwork(network);
      refresh();
    }
  }, [network]);

  const memoizedValue = useMemo(
    () => ({
      isLoading,
      error,
      totalRewards,
      refresh,
      redeemRewards: async () => {
        throw new Error('Not implemented');
      },
      redeemAllRewards: async () => {
        throw new Error('Not implemented');
      },
    }),
    [isLoading, error, totalRewards, network],
  );

  return <RewardsContext.Provider value={memoizedValue}>{children}</RewardsContext.Provider>;
};

export const useRewardsContext = () => useContext<TRewardsContext>(RewardsContext);

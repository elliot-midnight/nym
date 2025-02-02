import React, { useContext, useEffect, useState } from 'react';
import { Alert, Box, Button, CircularProgress } from '@mui/material';
import { useSnackbar } from 'notistack';
import { Fee, NymCard } from '../../components';
import { useCheckOwnership } from '../../hooks/useCheckOwnership';
import { AppContext } from '../../context/main';
import { unbond, vestingUnbond } from '../../requests';
import { PageLayout } from '../../layouts';

export const Unbond = () => {
  const [isLoading, setIsLoading] = useState(false);
  const { checkOwnership, ownership } = useCheckOwnership();
  const { userBalance, getBondDetails } = useContext(AppContext);

  const { enqueueSnackbar } = useSnackbar();

  useEffect(() => {
    const initialiseForm = async () => {
      await checkOwnership();
    };
    initialiseForm();
  }, [ownership.hasOwnership, checkOwnership]);

  return (
    <PageLayout>
      <NymCard title="Unbond" subheader="Unbond a mixnode or gateway" noPadding>
        {ownership?.hasOwnership ? (
          <>
            <Alert
              severity="info"
              data-testid="bond-noded"
              action={
                <Button
                  data-testid="un-bond"
                  disabled={isLoading}
                  onClick={async () => {
                    setIsLoading(true);
                    try {
                      if (ownership.vestingPledge) {
                        await vestingUnbond(ownership.nodeType!);
                      } else {
                        await unbond(ownership.nodeType!);
                      }
                    } catch (e) {
                      enqueueSnackbar(`Failed to unbond ${ownership.nodeType}}`, { variant: 'error' });
                    } finally {
                      await getBondDetails();
                      await checkOwnership();
                      await userBalance.fetchBalance();
                      setIsLoading(false);
                    }
                  }}
                  color="inherit"
                >
                  Unbond
                </Button>
              }
              sx={{ m: 2 }}
            >
              {`Looks like you already have a ${ownership.nodeType} bonded.`}
            </Alert>

            <Box sx={{ p: 3 }}>
              <Fee feeType="UnbondMixnode" />
            </Box>
          </>
        ) : (
          <Alert severity="info" sx={{ m: 3 }} data-testid="no-bond">
            You do not currently have a bonded node
          </Alert>
        )}
        {isLoading && (
          <Box
            sx={{
              display: 'flex',
              justifyContent: 'center',
              p: 3,
              pt: 0,
            }}
          >
            <CircularProgress size={48} />
          </Box>
        )}
      </NymCard>
    </PageLayout>
  );
};

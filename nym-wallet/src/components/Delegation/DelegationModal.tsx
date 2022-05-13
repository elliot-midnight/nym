import React from 'react';
import { Box, Button, Link, Modal, Typography } from '@mui/material';

const style = {
  position: 'absolute' as 'absolute',
  top: '50%',
  left: '50%',
  transform: 'translate(-50%, -50%)',
  width: 500,
  bgcolor: 'background.paper',
  boxShadow: 24,
  borderRadius: '16px',
  p: 4,
};

export type ActionType = 'delegate' | 'undelegate' | 'redeem' | 'redeem-all';

const actionToHeader = (action: ActionType): string => {
  // eslint-disable-next-line default-case
  switch (action) {
    case 'redeem':
      return 'Rewards redeemed successfully';
    case 'redeem-all':
      return 'All rewards redeemed successfully';
    case 'delegate':
      return 'Delegation complete';
    case 'undelegate':
      return 'Undelegation complete';
  }
  return 'Oh no! Something went wrong!';
};

export const DelegationModal: React.FC<{
  status: 'success' | 'error';
  action: ActionType;
  message: string;
  recipient: string;
  balance: string;
  transactionUrl: string;
  open: boolean;
  onClose?: () => void;
}> = ({ status, action, message, recipient, balance, transactionUrl, open, onClose, children }) => {
  if (status === 'error') {
    return (
      <Modal open={open} onClose={onClose}>
        <Box sx={style} textAlign="center">
          <Typography color={(theme) => theme.palette.error.main} mb={1}>
            Oh no! Something went wrong...
          </Typography>
          <Typography my={5}>{message}</Typography>
          {children}
          <Button variant="contained" onClick={onClose}>
            Close
          </Button>
        </Box>
      </Modal>
    );
  }
  return (
    <Modal open={open} onClose={onClose}>
      <Box sx={style} textAlign="center">
        <Typography color={(theme) => theme.palette.success.main} mb={1}>
          {actionToHeader(action)}
        </Typography>
        <Typography mb={3}>{message}</Typography>

        <Typography mb={1} fontSize="small" color={(theme) => theme.palette.text.secondary}>
          Recipient: {recipient}
        </Typography>
        <Typography mb={1} fontSize="small" color={(theme) => theme.palette.text.secondary}>
          Your current balance: {balance}
        </Typography>
        <Typography mb={1} fontSize="small" color={(theme) => theme.palette.text.secondary}>
          Check the transaction hash{' '}
          <Link href={transactionUrl} target="_blank">
            here
          </Link>
        </Typography>
        {children}
        <Button variant="contained" sx={{ mt: 3 }} size="large" onClick={onClose}>
          Finish
        </Button>
      </Box>
    </Modal>
  );
};

import { useEffect } from 'react';
import { useTonWallet } from '@tonconnect/ui-react';
import { api } from '../lib/api';
import { useTelegram } from './useTelegram';

export function useWalletSync() {
    const wallet = useTonWallet();
    const { user } = useTelegram();

    useEffect(() => {
        if (user && wallet) {
            const rawAddress = wallet.account.address;
            // userFriendlyAddress is usually better for display, but raw or userFriendly can be stored.
            // Let's store the raw address (0:...) as it's standard.
            // TonConnect wallet object has 'account.address' which is raw.

            console.log('Syncing wallet address:', rawAddress);
            api.post('/users/wallet', { address: rawAddress })
                .catch(err => console.error('Failed to sync wallet address:', err));
        }
    }, [user, wallet]);
}

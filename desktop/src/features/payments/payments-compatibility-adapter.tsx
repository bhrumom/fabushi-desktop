import React, { useMemo, useState } from 'react';
import type { MahayanaHostTransport } from '../../../../frontend/apps/web/src/lib/mahayana-host/transport';
import { FabButton, FabSurface } from '../../ui/primitives/fab-primitives';
import CompatibilityFeatureFrame from '../compatibility/compatibility-feature-frame';
import { useCompatibilityMessagingRuntime } from '../compatibility/use-compatibility-messaging-runtime';
import styles from '../compatibility/compatibility-data.module.css';

function money(currency: string, amountMinor: number): string {
  return `${currency.toUpperCase()} ${(amountMinor / 100).toFixed(2)}`;
}

export default function PaymentsCompatibilityAdapter({
  transport,
  onClose,
}: {
  readonly transport: MahayanaHostTransport;
  readonly onClose(): void;
}) {
  const runtime = useCompatibilityMessagingRuntime(transport, { includeWallet: true });
  const [busyOrderId, setBusyOrderId] = useState<string | null>(null);
  const invoiceById = useMemo(() => new Map(runtime.invoices.map((invoice) => [invoice.id, invoice] as const)), [runtime.invoices]);
  const balances = Object.entries(runtime.wallet?.balancesMinor ?? {});

  async function refund(orderId: string) {
    setBusyOrderId(orderId);
    try {
      await runtime.client.refundOrder(orderId);
    } finally {
      setBusyOrderId(null);
    }
  }

  return <CompatibilityFeatureFrame
    title="Payments"
    description="Wallet, Invoice, Order and refund state stays in the messaging ledger compatibility domain."
    onClose={onClose}
  >
    <div className={styles.toolbar}>
      <span className={styles.status}>Fabushi Wallet</span>
      <FabButton type="button" variant="ghost" onClick={() => void runtime.refresh()}>Refresh</FabButton>
    </div>
    <div className={styles.balanceGrid}>
      {balances.map(([currency, amount]) => <FabSurface key={currency} className={styles.balance} elevated>
        <small>{currency.toUpperCase()}</small><strong>{money(currency, amount)}</strong>
      </FabSurface>)}
      {!balances.length ? <FabSurface className={styles.balance}><small>Balance</small><strong>No balance</strong></FabSurface> : null}
    </div>
    <div className={styles.list}>
      {[...runtime.orders].sort((a,b) => b.updatedAtMs - a.updatedAtMs).map((order) => {
        const invoice = invoiceById.get(order.invoiceId);
        const refundable = invoice?.sellerId === runtime.client.actorId && order.status === 'paid';
        return <FabSurface className={styles.row} key={order.id}>
          <span className={styles.copy}>
            <strong>{invoice?.title ?? order.invoiceId}</strong>
            <small>{money(order.amount.currency, order.amount.amountMinor)} · {order.status}</small>
          </span>
          <span />
          {refundable ? <FabButton type="button" variant="danger" disabled={busyOrderId === order.id} onClick={() => void refund(order.id)}>
            {busyOrderId === order.id ? 'Refunding…' : 'Refund'}
          </FabButton> : <span className={styles.meta}>{new Date(order.updatedAtMs).toLocaleString()}</span>}
        </FabSurface>;
      })}
    </div>
    {runtime.error ? <div className={styles.error} role="alert">{runtime.error}</div> : null}
  </CompatibilityFeatureFrame>;
}

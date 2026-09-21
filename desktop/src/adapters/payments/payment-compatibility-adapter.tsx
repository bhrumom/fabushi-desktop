import { ShoppingBag, WalletCards } from 'lucide-react';
import React from 'react';
import type {
  MessagingInvoice,
  MessagingLedgerEntry,
  MessagingOrder,
  MessagingWalletAccount,
} from '../../selfhosted-messaging-client-v2';
import type { CompatibilitySection } from '../../agent-workspace/messenger-compatibility-adapter';
import { FabButton } from '../../ui/primitives/fab-primitives';
import styles from '../../messaging-shell.module.css';
import extra from '../legacy-messaging/legacy-messaging-shell.module.css';

export type PaymentUiState = {
  account: MessagingWalletAccount | null;
  entries: MessagingLedgerEntry[];
  orders: MessagingOrder[];
  invoices: MessagingInvoice[];
  actorId: string;
};

function moneyLabel(currency: string, amountMinor: number): string {
  return `${currency.toUpperCase()} ${(amountMinor / 100).toFixed(2)}`;
}

export function PaymentOverview({
  payment,
  onInvoice,
  onRefund,
  compact = false,
}: {
  payment: PaymentUiState;
  onInvoice: () => void;
  onRefund: (orderId: string) => void;
  compact?: boolean;
}) {
  const balances = Object.entries(payment.account?.balancesMinor ?? {});
  const invoiceById = new Map(payment.invoices.map((invoice) => [invoice.id, invoice]));
  const orders = [...payment.orders].sort((left, right) => right.updatedAtMs - left.updatedAtMs);
  return <div className={compact ? extra.paymentCompact : extra.paymentOverview}>
    <div className={extra.walletCard}><WalletCards size={24} /><div><strong>Fabushi Wallet</strong><small>{balances.length ? balances.map(([currency, amount]) => moneyLabel(currency, amount)).join(' · ') : '暂无余额'}</small></div></div>
    <FabButton variant="primary" className={styles.primaryButton} onClick={onInvoice}><ShoppingBag size={17} />创建账单</FabButton>
    {!compact ? <>
      <section className={extra.paymentSection}><strong>订单</strong>{orders.length ? orders.map((order) => {
        const invoice = invoiceById.get(order.invoiceId);
        const sellerOwned = invoice?.sellerId === payment.actorId;
        return <div className={extra.paymentRow} key={order.id}><div><strong>{invoice?.title ?? order.invoiceId}</strong><small>{moneyLabel(order.amount.currency, order.amount.amountMinor)} · {order.status}</small></div>{sellerOwned && order.status === 'paid' ? <FabButton onClick={() => onRefund(order.id)}>退款</FabButton> : null}</div>;
      }) : <small>暂无订单</small>}</section>
      <section className={extra.paymentSection}><strong>最近流水</strong>{payment.entries.length ? payment.entries.slice(0, 12).map((entry) => <div className={extra.paymentRow} key={entry.id}><div><strong>{entry.kind}</strong><small>{entry.reference ?? entry.id}</small></div><span>{moneyLabel(entry.amount.currency, entry.amount.amountMinor)}</span></div>) : <small>暂无流水</small>}</section>
    </> : null}
  </div>;
}

export default function PaymentCompatibilityAdapter({
  section,
  children,
}: {
  readonly section: CompatibilitySection;
  readonly children: React.ReactNode;
}) {
  return section === 'payments' ? <>{children}</> : null;
}

import React from 'react';
import { ShoppingBag, WalletCards } from 'lucide-react';
import type { MessagingInvoice, MessagingLedgerEntry, MessagingOrder, MessagingWalletAccount } from '../../selfhosted-messaging-client-v2';
import { FabButton, FabDialog, FabDialogActions, FabIconButton, FabInput, FabSurface } from '../../ui/primitives/fab-primitives';
import styles from '../../messaging-shell.module.css';
import type { InvoiceDialogState } from '../compatibility/compatibility-model';
import { X } from 'lucide-react';
import extra from '../../agent-workspace/agent-root-shell.module.css';

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

export function PaymentOverview({ payment, onInvoice, onRefund, compact = false }: {
  payment: PaymentUiState;
  onInvoice: () => void;
  onRefund: (orderId: string) => void;
  compact?: boolean;
}) {
  const balances = Object.entries(payment.account?.balancesMinor ?? {});
  const invoiceById = new Map(payment.invoices.map((invoice) => [invoice.id, invoice]));
  const orders = [...payment.orders].sort((left, right) => right.updatedAtMs - left.updatedAtMs);
  return <div className={compact ? extra.paymentCompact : extra.paymentOverview}>
    <FabSurface className={extra.walletCard} elevated><WalletCards size={24} /><div><strong>Fabushi Wallet</strong><small>{balances.length ? balances.map(([currency, amount]) => moneyLabel(currency, amount)).join(' · ') : '暂无余额'}</small></div></FabSurface>
    <FabButton variant="primary" onClick={onInvoice}><ShoppingBag size={17} />创建账单</FabButton>
    {!compact ? <>
      <section className={extra.paymentSection}><strong>订单</strong>{orders.length ? orders.map((order) => {
        const invoice = invoiceById.get(order.invoiceId);
        const sellerOwned = invoice?.sellerId === payment.actorId;
        return <div className={extra.paymentRow} key={order.id}><div><strong>{invoice?.title ?? order.invoiceId}</strong><small>{moneyLabel(order.amount.currency, order.amount.amountMinor)} · {order.status}</small></div>{sellerOwned && order.status === 'paid' ? <FabButton variant="ghost" onClick={() => onRefund(order.id)}>退款</FabButton> : null}</div>;
      }) : <small>暂无订单</small>}</section>
      <section className={extra.paymentSection}><strong>最近流水</strong>{payment.entries.length ? payment.entries.slice(0, 12).map((entry) => <div className={extra.paymentRow} key={entry.id}><div><strong>{entry.kind}</strong><small>{entry.reference ?? entry.id}</small></div><span>{moneyLabel(entry.amount.currency, entry.amount.amountMinor)}</span></div>) : <small>暂无流水</small>}</section>
    </> : null}
  </div>;
}

export function PaymentsCompatibilityWorkspace(props: { payment: PaymentUiState; onInvoice(): void; onRefund(orderId: string): void }) {
  return <FabSurface className={styles.featureWorkspace} elevated data-compatibility-feature="payments">
    <WalletCards size={54} /><h2>Fabushi Pay</h2>
    <p>Invoice、Order、退款与 settlement 由 Rust 账本结算；此 adapter 只负责兼容产品表面。</p>
    <PaymentOverview payment={props.payment} onInvoice={props.onInvoice} onRefund={props.onRefund} />
  </FabSurface>;
}


export function PaymentInvoiceDialog({
  dialog,
  onChange,
  onClose,
  onSave,
}: {
  dialog: Exclude<InvoiceDialogState, null>;
  onChange: React.Dispatch<React.SetStateAction<InvoiceDialogState>>;
  onClose: () => void;
  onSave: () => void;
}) {
  const amount = Number(dialog.amount);
  const validAmount = Number.isFinite(amount) && amount > 0;
  return <FabDialog label="发送账单" onClose={onClose}>
    <header><div><strong>发送账单</strong><small>Fabushi Pay · USD</small></div><FabIconButton label="关闭" onClick={onClose}><X size={17} /></FabIconButton></header>
    <label><span>账单名称</span><FabInput autoFocus data-testid="invoice-title-input" value={dialog.title} onChange={(event) => onChange((current) => current ? { ...current, title: event.target.value } : current)} placeholder="订单" /></label>
    <label><span>金额（USD）</span><FabInput data-testid="invoice-amount-input" inputMode="decimal" value={dialog.amount} onChange={(event) => onChange((current) => current ? { ...current, amount: event.target.value } : current)} placeholder="9.99" /></label>
    <FabDialogActions><FabButton type="button" variant="ghost" onClick={onClose}>取消</FabButton><FabButton type="button" variant="primary" disabled={!dialog.title.trim() || !validAmount} onClick={onSave}>创建账单</FabButton></FabDialogActions>
  </FabDialog>;
}

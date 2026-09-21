import React from 'react';
import { ShoppingBag, WalletCards, X } from 'lucide-react';
import type { MessagingInvoice, MessagingLedgerEntry, MessagingOrder, MessagingWalletAccount } from '../../selfhosted-messaging-client-v2';
import type { InvoiceDialogState } from '../../adapters/compatibility/compatibility-model';
import styles from '../../messaging-shell.module.css';
import extra from '../../adapters/compatibility/compatibility.module.css';

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

export function PaymentCompatibilityAdapter({ payment, onInvoice, onRefund, compact = false }: { payment: PaymentUiState; onInvoice: () => void; onRefund: (orderId: string) => void; compact?: boolean }) {
  const balances = Object.entries(payment.account?.balancesMinor ?? {});
  const invoiceById = new Map(payment.invoices.map((invoice) => [invoice.id, invoice]));
  const orders = [...payment.orders].sort((left, right) => right.updatedAtMs - left.updatedAtMs);
  return <div className={compact ? extra.paymentCompact : extra.paymentOverview}>
    <div className={extra.walletCard}><WalletCards size={24} /><div><strong>Fabushi Wallet</strong><small>{balances.length ? balances.map(([currency, amount]) => moneyLabel(currency, amount)).join(' · ') : '暂无余额'}</small></div></div>
    <button type="button" className={styles.primaryButton} onClick={onInvoice}><ShoppingBag size={17} />创建账单</button>
    {!compact ? <>
      <section className={extra.paymentSection}><strong>订单</strong>{orders.length ? orders.map((order) => {
        const invoice = invoiceById.get(order.invoiceId);
        const sellerOwned = invoice?.sellerId === payment.actorId;
        return <div className={extra.paymentRow} key={order.id}><div><strong>{invoice?.title ?? order.invoiceId}</strong><small>{moneyLabel(order.amount.currency, order.amount.amountMinor)} · {order.status}</small></div>{sellerOwned && order.status === 'paid' ? <button type="button" onClick={() => onRefund(order.id)}>退款</button> : null}</div>;
      }) : <small>暂无订单</small>}</section>
      <section className={extra.paymentSection}><strong>最近流水</strong>{payment.entries.length ? payment.entries.slice(0, 12).map((entry) => <div className={extra.paymentRow} key={entry.id}><div><strong>{entry.kind}</strong><small>{entry.reference ?? entry.id}</small></div><span>{moneyLabel(entry.amount.currency, entry.amount.amountMinor)}</span></div>) : <small>暂无流水</small>}</section>
    </> : null}
  </div>;
}


export function InvoiceCompatibilityDialog({ dialog, onChange, onClose, onSave }: { dialog: Exclude<InvoiceDialogState, null>; onChange: React.Dispatch<React.SetStateAction<InvoiceDialogState>>; onClose: () => void; onSave: () => void }) {
  const amount = Number(dialog.amount);
  const validAmount = Number.isFinite(amount) && amount > 0;
  return <div className={styles.backdrop} onMouseDown={onClose}><section className={styles.dialog} onMouseDown={(event) => event.stopPropagation()}>
    <header><div><strong>发送账单</strong><small>Fabushi Pay · USD</small></div><button type="button" onClick={onClose}><X size={17} /></button></header>
    <label><span>账单名称</span><input autoFocus data-testid="invoice-title-input" value={dialog.title} onChange={(event) => onChange((current) => current ? { ...current, title: event.target.value } : current)} placeholder="订单" /></label>
    <label><span>金额（USD）</span><input data-testid="invoice-amount-input" inputMode="decimal" value={dialog.amount} onChange={(event) => onChange((current) => current ? { ...current, amount: event.target.value } : current)} placeholder="9.99" /></label>
    <footer><button type="button" onClick={onClose}>取消</button><button type="button" className={styles.primaryButton} disabled={!dialog.title.trim() || !validAmount} onClick={onSave}>创建账单</button></footer>
  </section></div>;
}


import React from 'react';
import FabAvatar, { type FabAvatarProps } from '../avatar/fab-avatar';
import styles from './fab-primitives.module.css';

function classes(...values: Array<string | undefined | false>): string {
  return values.filter(Boolean).join(' ');
}

export function FabButton({
  variant = 'default',
  className,
  ...props
}: React.ButtonHTMLAttributes<HTMLButtonElement> & {
  readonly variant?: 'default' | 'primary' | 'danger' | 'ghost' | 'bare';
}) {
  return <button
    {...props}
    className={classes(variant === 'bare' ? styles.bareButton : styles.button, className)}
    data-variant={variant}
  />;
}

export function FabIconButton({
  label,
  className,
  children,
  ...props
}: React.ButtonHTMLAttributes<HTMLButtonElement> & {
  readonly label: string;
}) {
  return <button
    {...props}
    type={props.type ?? 'button'}
    aria-label={label}
    title={props.title ?? label}
    className={classes(styles.iconButton, className)}
  >{children}</button>;
}

export function FabInput({
  variant = 'default',
  className,
  ...props
}: React.InputHTMLAttributes<HTMLInputElement> & {
  readonly variant?: 'default' | 'bare';
}) {
  return <input {...props} className={classes(variant === 'bare' ? styles.bareInput : styles.input, className)} />;
}

export function FabSelect({
  variant = 'default',
  className,
  children,
  ...props
}: React.SelectHTMLAttributes<HTMLSelectElement> & {
  readonly variant?: 'default' | 'bare';
}) {
  return <select {...props} className={classes(variant === 'bare' ? styles.bareSelect : styles.select, className)}>{children}</select>;
}

export function FabSwitch({
  className,
  ...props
}: Omit<React.InputHTMLAttributes<HTMLInputElement>, 'type' | 'role'>) {
  return <input
    {...props}
    type="checkbox"
    role="switch"
    className={classes(styles.switch, className)}
  />;
}

export function FabSurface({
  className,
  elevated = false,
  children,
  ...props
}: React.HTMLAttributes<HTMLDivElement> & { readonly elevated?: boolean }) {
  return <div {...props} className={classes(styles.surface, className)} data-elevated={elevated || undefined}>{children}</div>;
}

export function FabBadge({
  tone = 'neutral',
  className,
  children,
  ...props
}: React.HTMLAttributes<HTMLSpanElement> & {
  readonly tone?: 'neutral' | 'accent' | 'success' | 'warning' | 'danger';
}) {
  return <span {...props} className={classes(styles.badge, className)} data-tone={tone}>{children}</span>;
}

export function FabSpinner({
  label = 'Loading',
  size = 16,
  className,
}: {
  readonly label?: string;
  readonly size?: number;
  readonly className?: string;
}) {
  return <span
    className={classes(styles.spinner, className)}
    style={{ '--fab-spinner-size': `${Math.max(12, Math.round(size))}px` } as React.CSSProperties}
    role="status"
    aria-label={label}
  />;
}

export function FabTooltip({
  label,
  children,
  className,
}: {
  readonly label: string;
  readonly children: React.ReactNode;
  readonly className?: string;
}) {
  return <span className={classes(styles.tooltip, className)} data-tooltip={label}>{children}</span>;
}

export function FabPopover({
  open,
  anchor,
  children,
  className,
  align = 'start',
}: {
  readonly open: boolean;
  readonly anchor: React.ReactNode;
  readonly children: React.ReactNode;
  readonly className?: string;
  readonly align?: 'start' | 'end';
}) {
  return <span className={classes(styles.popoverRoot, className)}>
    {anchor}
    {open ? <span className={styles.popover} data-align={align} role="presentation">{children}</span> : null}
  </span>;
}

export function FabMenu({
  label,
  children,
  className,
}: {
  readonly label: string;
  readonly children: React.ReactNode;
  readonly className?: string;
}) {
  return <div className={classes(styles.menu, className)} role="menu" aria-label={label}>{children}</div>;
}

export function FabMenuItem({
  className,
  children,
  ...props
}: React.ButtonHTMLAttributes<HTMLButtonElement>) {
  return <button {...props} type={props.type ?? 'button'} role="menuitem" className={classes(styles.menuItem, className)}>{children}</button>;
}

export function FabDialog({
  label,
  onClose,
  onSubmit,
  children,
  surfaceClassName,
}: {
  readonly label: string;
  readonly onClose: () => void;
  readonly onSubmit?: React.FormEventHandler<HTMLFormElement>;
  readonly children: React.ReactNode;
  readonly surfaceClassName?: string;
}) {
  const surface = onSubmit
    ? <form className={classes(styles.dialogSurface, surfaceClassName)} onSubmit={onSubmit}>{children}</form>
    : <div className={classes(styles.dialogSurface, surfaceClassName)}>{children}</div>;
  return <div
    role="dialog"
    aria-modal="true"
    aria-label={label}
    className={styles.dialogOverlay}
    onMouseDown={(event) => {
      if (event.target === event.currentTarget) onClose();
    }}
  >
    {surface}
  </div>;
}

export function FabDialogActions({ children }: { readonly children: React.ReactNode }) {
  return <div className={styles.dialogActions}>{children}</div>;
}

export function FabAvatarPrimitive(props: FabAvatarProps) {
  return <FabAvatar {...props} />;
}

export { FabAvatar };

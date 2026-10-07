export interface ChildSelectionSubscriptionSource<TDestination> {
  getSnapshot(): { readonly selected: TDestination | null };
  subscribe(listener: () => void): () => void;
}

export interface ChildPaginationScopeSink {
  setScope(scopeKey: string | null): void;
}

export function bindConversationChildSelectionToPagination<TDestination>(
  selection: ChildSelectionSubscriptionSource<TDestination>,
  pagination: ChildPaginationScopeSink,
  scopeKey: (destination: TDestination) => string,
  onSelectionChange: () => void = () => {}
): () => void {
  const sync = () => {
    const destination = selection.getSnapshot().selected;
    pagination.setScope(destination == null ? null : scopeKey(destination));
    onSelectionChange();
  };
  sync();
  return selection.subscribe(sync);
}

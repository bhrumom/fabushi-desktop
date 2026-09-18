import { Component, type ErrorInfo, type ReactNode } from 'react';
import { ErrorBoundarySurface } from '../recovered/features/error-boundary/view';

interface State {
  error: Error | null;
  componentStack: string | null;
}

export class RootErrorBoundary extends Component<{children:ReactNode}, State> {
  state:State={error:null,componentStack:null};
  static getDerivedStateFromError(error:Error):State {
    return {error,componentStack:null};
  }
  componentDidCatch(_error:Error,info:ErrorInfo) {
    this.setState({componentStack:info.componentStack??null});
  }
  render() {
    if (!this.state.error) return this.props.children;
    return <ErrorBoundarySurface
      error={this.state.error}
      componentStack={this.state.componentStack}
      labels={{
        title:'Something went wrong',
        detail:'Fabushi hit an unexpected renderer error.',
        reload:'Reload',
        copyError:'Copy error',
        copied:'Copied'
      }}
    />;
  }
}

import { writable, type Writable } from 'svelte/store';

export interface UIState {
  isLoading: boolean;
  isSidebarOpen: boolean;
  activeModal: string | null;
  error: string | null;
}

const initialState: UIState = {
  isLoading: false,
  isSidebarOpen: true,
  activeModal: null,
  error: null,
};

export const uiState: Writable<UIState> = writable(initialState);

export function setLoading(loading: boolean): void {
  uiState.update((state) => ({ ...state, isLoading: loading }));
}

export function toggleSidebar(): void {
  uiState.update((state) => ({ ...state, isSidebarOpen: !state.isSidebarOpen }));
}

export function openModal(modalName: string): void {
  uiState.update((state) => ({ ...state, activeModal: modalName }));
}

export function closeModal(): void {
  uiState.update((state) => ({ ...state, activeModal: null }));
}

export function setError(error: string | null): void {
  uiState.update((state) => ({ ...state, error }));
}

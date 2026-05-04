export type UiState = {
  detailsFocused: boolean;
  monitoringPaused: boolean;
};

export const defaultUiState: UiState = {
  detailsFocused: false,
  monitoringPaused: false,
};

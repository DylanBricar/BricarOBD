import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import LiveDataToolbar from './LiveDataToolbar';

const mockT = (key: string) => key;

describe('LiveDataToolbar', () => {
  const defaultProps = {
    search: '',
    onSearchChange: vi.fn(),
    showPidSelector: false,
    onTogglePidSelector: vi.fn(),
    selectedPidsCount: 5,
    totalPidsCount: 20,
    refreshRate: 1000,
    onRefreshRateChange: vi.fn(),
    isActive: false,
    onTogglePolling: vi.fn(),
    isRecording: false,
    recordingDuration: 0,
    onStartRecording: vi.fn(),
    onStopRecording: vi.fn(),
    pidDataSize: 10,
    onExportCSV: vi.fn(),
    t: mockT,
  };

  it('renders search input with placeholder', () => {
    render(<LiveDataToolbar {...defaultProps} />);
    const searchInput = screen.getByPlaceholderText('liveData.search');
    expect(searchInput).toBeDefined();
  });

  it('calls onSearchChange when search input changes', () => {
    const onSearchChange = vi.fn();
    render(<LiveDataToolbar {...defaultProps} onSearchChange={onSearchChange} />);
    const searchInput = screen.getByPlaceholderText('liveData.search') as HTMLInputElement;
    fireEvent.change(searchInput, { target: { value: 'test' } });
    expect(onSearchChange).toHaveBeenCalledWith('test');
  });

  it('renders PID count display', () => {
    render(<LiveDataToolbar {...defaultProps} selectedPidsCount={5} totalPidsCount={20} />);
    expect(screen.getByText('5/20')).toBeDefined();
  });

  it('calls onTogglePidSelector when PID selector button is clicked', () => {
    const onTogglePidSelector = vi.fn();
    render(<LiveDataToolbar {...defaultProps} onTogglePidSelector={onTogglePidSelector} />);
    const selectorButton = screen.getByText('5/20');
    expect(selectorButton).toBeDefined();
    fireEvent.click(selectorButton!);
    expect(onTogglePidSelector).toHaveBeenCalled();
  });

  it('renders refresh rate dropdown', () => {
    render(<LiveDataToolbar {...defaultProps} />);
    const refreshSelect = screen.getByRole('combobox');
    expect(refreshSelect).toBeDefined();
  });

  it('calls onRefreshRateChange when refresh rate is changed', () => {
    const onRefreshRateChange = vi.fn();
    render(<LiveDataToolbar {...defaultProps} onRefreshRateChange={onRefreshRateChange} />);
    const refreshSelect = screen.getByRole('combobox') as HTMLSelectElement;
    fireEvent.change(refreshSelect, { target: { value: '500' } });
    expect(onRefreshRateChange).toHaveBeenCalledWith(500);
  });

  it('shows pause button when isActive is true', () => {
    render(<LiveDataToolbar {...defaultProps} isActive={true} />);
    expect(screen.getByText('liveData.pause')).toBeDefined();
  });

  it('shows start button when isActive is false', () => {
    render(<LiveDataToolbar {...defaultProps} isActive={false} />);
    expect(screen.getByText('liveData.start')).toBeDefined();
  });

  it('calls onTogglePolling when play/pause button is clicked', () => {
    const onTogglePolling = vi.fn();
    render(<LiveDataToolbar {...defaultProps} onTogglePolling={onTogglePolling} isActive={true} />);
    const pauseButton = screen.getByText('liveData.pause');
    expect(pauseButton).toBeDefined();
    fireEvent.click(pauseButton!);
    expect(onTogglePolling).toHaveBeenCalled();
  });

  it('shows record button when not recording', () => {
    render(<LiveDataToolbar {...defaultProps} isRecording={false} />);
    expect(screen.getByText('liveData.record')).toBeDefined();
  });

  it('shows stop with duration when recording', () => {
    render(<LiveDataToolbar {...defaultProps} isRecording={true} recordingDuration={15} />);
    expect(screen.getByText(/liveData.stop.*15s/)).toBeDefined();
  });

  it('calls onStartRecording when record button is clicked', () => {
    const onStartRecording = vi.fn();
    render(<LiveDataToolbar {...defaultProps} onStartRecording={onStartRecording} isRecording={false} />);
    const recordButton = screen.getByText('liveData.record');
    expect(recordButton).toBeDefined();
    fireEvent.click(recordButton!);
    expect(onStartRecording).toHaveBeenCalled();
  });

  it('calls onStopRecording when stop button is clicked', () => {
    const onStopRecording = vi.fn();
    render(<LiveDataToolbar {...defaultProps} onStopRecording={onStopRecording} isRecording={true} recordingDuration={5} />);
    const stopButton = screen.getByText(/liveData.stop/);
    expect(stopButton).toBeDefined();
    fireEvent.click(stopButton!);
    expect(onStopRecording).toHaveBeenCalled();
  });

  it('CSV export button is enabled when pidDataSize > 0', () => {
    render(<LiveDataToolbar {...defaultProps} pidDataSize={10} />);
    const csvButton = screen.getByText('CSV') as HTMLButtonElement;
    expect(csvButton).toBeDefined();
    expect(csvButton!.disabled).toBe(false);
  });

  it('CSV export button is disabled when pidDataSize is 0', () => {
    render(<LiveDataToolbar {...defaultProps} pidDataSize={0} />);
    const csvButton = screen.getByText('CSV') as HTMLButtonElement;
    expect(csvButton).toBeDefined();
    expect(csvButton!.disabled).toBe(true);
  });

  it('calls onExportCSV when CSV button is clicked', () => {
    const onExportCSV = vi.fn();
    render(<LiveDataToolbar {...defaultProps} onExportCSV={onExportCSV} pidDataSize={10} />);
    const csvButton = screen.getByText('CSV');
    expect(csvButton).toBeDefined();
    fireEvent.click(csvButton!);
    expect(onExportCSV).toHaveBeenCalled();
  });
});

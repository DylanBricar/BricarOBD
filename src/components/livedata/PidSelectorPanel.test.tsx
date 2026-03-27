import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import PidSelectorPanel from './PidSelectorPanel';

const mockT = (key: string) => key;

interface PidValue {
  pid: number;
  name: string;
  value: number;
  unit: string;
}

describe('PidSelectorPanel', () => {
  const mockPidData = new Map<number, PidValue>([
    [0x01, { pid: 0x01, name: 'Engine Load', value: 45, unit: '%' }],
    [0x02, { pid: 0x02, name: 'Engine Temperature', value: 85, unit: '°C' }],
    [0x03, { pid: 0x03, name: 'Fuel Pressure', value: 300, unit: 'kPa' }],
  ]);

  const defaultProps = {
    pidData: mockPidData,
    selectedPids: new Set<number>([0x01]),
    onSelectedPidsChange: vi.fn(),
    sortedPidValues: Array.from(mockPidData.values()),
    t: mockT,
  };

  it('renders header with title and count', () => {
    render(<PidSelectorPanel {...defaultProps} />);
    expect(screen.getByText(/liveData\.selectParameters/)).toBeDefined();
    expect(screen.getByText(/1\/3/)).toBeDefined();
  });

  it('renders Select All button', () => {
    render(<PidSelectorPanel {...defaultProps} />);
    expect(screen.getByText('liveData.selectAll')).toBeDefined();
  });

  it('renders Select None button', () => {
    render(<PidSelectorPanel {...defaultProps} />);
    expect(screen.getByText('liveData.selectNone')).toBeDefined();
  });

  it('calls onSelectedPidsChange with all PIDs when Select All is clicked', () => {
    const onSelectedPidsChange = vi.fn();
    render(<PidSelectorPanel {...defaultProps} onSelectedPidsChange={onSelectedPidsChange} />);
    const selectAllButton = screen.getByText('liveData.selectAll');
    expect(selectAllButton).toBeDefined();
    fireEvent.click(selectAllButton!);
    expect(onSelectedPidsChange).toHaveBeenCalledWith(new Set([0x01, 0x02, 0x03]));
  });

  it('calls onSelectedPidsChange with empty set when Select None is clicked', () => {
    const onSelectedPidsChange = vi.fn();
    render(<PidSelectorPanel {...defaultProps} onSelectedPidsChange={onSelectedPidsChange} />);
    const selectNoneButton = screen.getByText('liveData.selectNone');
    expect(selectNoneButton).toBeDefined();
    fireEvent.click(selectNoneButton!);
    expect(onSelectedPidsChange).toHaveBeenCalledWith(new Set());
  });

  it('renders PID button for each PID', () => {
    render(<PidSelectorPanel {...defaultProps} />);
    expect(screen.getByText('Engine Load')).toBeDefined();
    expect(screen.getByText('Engine Temperature')).toBeDefined();
    expect(screen.getByText('Fuel Pressure')).toBeDefined();
  });

  it('shows checkbox div with bg-obd-accent when PID is selected', () => {
    render(<PidSelectorPanel {...defaultProps} selectedPids={new Set([0x01])} />);
    const engineLoadButton = screen.getByText('Engine Load').closest('button');
    expect(engineLoadButton).toBeDefined();
    const checkboxDiv = engineLoadButton!.querySelector('div[class*="bg-obd-accent"]');
    expect(checkboxDiv).toBeDefined();
  });

  it('shows checkbox div without bg-obd-accent when PID is not selected', () => {
    render(<PidSelectorPanel {...defaultProps} selectedPids={new Set()} />);
    const engineLoadButton = screen.getByText('Engine Load').closest('button');
    expect(engineLoadButton).toBeDefined();
    const checkboxDiv = engineLoadButton!.querySelector('div');
    expect(checkboxDiv).toBeDefined();
    expect(checkboxDiv!.className).not.toContain('bg-obd-accent');
  });

  it('toggles individual PID selection when clicked', () => {
    const onSelectedPidsChange = vi.fn();
    render(
      <PidSelectorPanel
        {...defaultProps}
        selectedPids={new Set([0x01])}
        onSelectedPidsChange={onSelectedPidsChange}
      />
    );
    const engineTempButton = screen.getByText('Engine Temperature');
    expect(engineTempButton).toBeDefined();
    fireEvent.click(engineTempButton!);
    expect(onSelectedPidsChange).toHaveBeenCalledWith(new Set([0x01, 0x02]));
  });

  it('removes PID from selection when already selected', () => {
    const onSelectedPidsChange = vi.fn();
    render(
      <PidSelectorPanel
        {...defaultProps}
        selectedPids={new Set([0x01, 0x02])}
        onSelectedPidsChange={onSelectedPidsChange}
      />
    );
    const engineLoadButton = screen.getByText('Engine Load');
    expect(engineLoadButton).toBeDefined();
    fireEvent.click(engineLoadButton!);
    expect(onSelectedPidsChange).toHaveBeenCalledWith(new Set([0x02]));
  });

  it('container has glass-card class', () => {
    const { container } = render(<PidSelectorPanel {...defaultProps} />);
    const glassCard = container.querySelector('.glass-card');
    expect(glassCard).toBeDefined();
  });

  it('container has space-y-2 class', () => {
    const { container } = render(<PidSelectorPanel {...defaultProps} />);
    const container_ = container.querySelector('[class*="space-y-2"]');
    expect(container_).toBeDefined();
  });

  it('container has max-h-28 for scrolling', () => {
    const { container } = render(<PidSelectorPanel {...defaultProps} />);
    const container_ = container.querySelector('[class*="max-h-28"]');
    expect(container_).toBeDefined();
  });

  it('updates count when selected PIDs change', () => {
    const { rerender } = render(<PidSelectorPanel {...defaultProps} selectedPids={new Set([0x01])} />);
    expect(screen.getByText(/1\/3/)).toBeDefined();
    rerender(<PidSelectorPanel {...defaultProps} selectedPids={new Set([0x01, 0x02, 0x03])} />);
    expect(screen.getByText(/3\/3/)).toBeDefined();
  });
});

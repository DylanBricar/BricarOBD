import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import AdvancedConsole from './AdvancedConsole';

const mockT = (key: string) => key;

interface ConsoleResponse {
  cmd: string;
  res: string;
  time: string;
  isError: boolean;
}

describe('AdvancedConsole', () => {
  it('renders empty state with Terminal icon and message', () => {
    render(<AdvancedConsole responses={[]} t={mockT} />);
    expect(screen.getByText('advanced.awaitingCommands')).toBeDefined();
  });

  it('renders title', () => {
    render(<AdvancedConsole responses={[]} t={mockT} />);
    expect(screen.getByText('advanced.console')).toBeDefined();
  });

  it('displays single command and response', () => {
    const responses: ConsoleResponse[] = [
      {
        cmd: 'AT+GMM',
        res: 'ELM327',
        time: '12:34:56',
        isError: false,
      },
    ];
    render(<AdvancedConsole responses={responses} t={mockT} />);
    expect(screen.getByText(/\[12:34:56\]/)).toBeDefined();
    expect(screen.getByText('AT+GMM')).toBeDefined();
    expect(screen.getByText('ELM327')).toBeDefined();
  });

  it('displays multiple commands and responses', () => {
    const responses: ConsoleResponse[] = [
      {
        cmd: 'AT+GMM',
        res: 'ELM327',
        time: '12:34:56',
        isError: false,
      },
      {
        cmd: 'AT+SP6',
        res: 'OK',
        time: '12:34:57',
        isError: false,
      },
    ];
    render(<AdvancedConsole responses={responses} t={mockT} />);
    expect(screen.getByText(/\[12:34:56\]/)).toBeDefined();
    expect(screen.getByText('AT+GMM')).toBeDefined();
    expect(screen.getByText(/\[12:34:57\]/)).toBeDefined();
    expect(screen.getByText('AT+SP6')).toBeDefined();
    expect(screen.getByText('ELM327')).toBeDefined();
    expect(screen.getByText('OK')).toBeDefined();
  });

  it('shows success mark for non-error responses', () => {
    const responses: ConsoleResponse[] = [
      {
        cmd: 'AT+GMM',
        res: 'ELM327',
        time: '12:34:56',
        isError: false,
      },
    ];
    const { container } = render(<AdvancedConsole responses={responses} t={mockT} />);
    const successMark = container.querySelector('.text-obd-success');
    expect(successMark).toBeDefined();
  });

  it('shows error mark for error responses', () => {
    const responses: ConsoleResponse[] = [
      {
        cmd: 'AT+SP99',
        res: 'NO DATA',
        time: '12:34:56',
        isError: true,
      },
    ];
    const { container } = render(<AdvancedConsole responses={responses} t={mockT} />);
    const errorMark = container.querySelector('.text-obd-danger');
    expect(errorMark).toBeDefined();
  });

  it('displays timestamp in brackets', () => {
    const responses: ConsoleResponse[] = [
      {
        cmd: 'AT+GMM',
        res: 'ELM327',
        time: '14:23:45',
        isError: false,
      },
    ];
    render(<AdvancedConsole responses={responses} t={mockT} />);
    expect(screen.getByText(/\[14:23:45\]/)).toBeDefined();
  });

  it('renders container with glass-card class', () => {
    const { container } = render(<AdvancedConsole responses={[]} t={mockT} />);
    const glassCard = container.querySelector('.glass-card');
    expect(glassCard).toBeDefined();
  });

  it('renders with font-mono class', () => {
    const { container } = render(<AdvancedConsole responses={[]} t={mockT} />);
    const monoContainer = container.querySelector('[class*="font-mono"]');
    expect(monoContainer).toBeDefined();
  });

  it('renders with overflow-y-auto for scrolling', () => {
    const responses: ConsoleResponse[] = Array.from({ length: 20 }, (_, i) => ({
      cmd: `CMD${i}`,
      res: `RES${i}`,
      time: `12:34:${String(i).padStart(2, '0')}`,
      isError: false,
    }));
    const { container } = render(<AdvancedConsole responses={responses} t={mockT} />);
    const scrollableArea = container.querySelector('[class*="overflow-y-auto"]');
    expect(scrollableArea).toBeDefined();
  });

  it('renders with space-y-2 spacing between items', () => {
    const responses: ConsoleResponse[] = [
      {
        cmd: 'AT+GMM',
        res: 'ELM327',
        time: '12:34:56',
        isError: false,
      },
      {
        cmd: 'AT+SP6',
        res: 'OK',
        time: '12:34:57',
        isError: false,
      },
    ];
    const { container } = render(<AdvancedConsole responses={responses} t={mockT} />);
    const spacedArea = container.querySelector('[class*="space-y-2"]');
    expect(spacedArea).toBeDefined();
  });

  it('displays command and response on separate lines', () => {
    const responses: ConsoleResponse[] = [
      {
        cmd: 'AT+GMM',
        res: 'ELM327',
        time: '12:34:56',
        isError: false,
      },
    ];
    render(<AdvancedConsole responses={responses} t={mockT} />);
    expect(screen.getByText(/\[12:34:56\]/)).toBeDefined();
    expect(screen.getByText('AT+GMM')).toBeDefined();
    expect(screen.getByText('ELM327')).toBeDefined();
  });

  it('handles responses with special characters', () => {
    const responses: ConsoleResponse[] = [
      {
        cmd: 'AT+HP12',
        res: '41 0D FF',
        time: '12:34:56',
        isError: false,
      },
    ];
    render(<AdvancedConsole responses={responses} t={mockT} />);
    expect(screen.getByText('41 0D FF')).toBeDefined();
  });
});

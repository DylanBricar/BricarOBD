/// Parse service ID from hex command (handles spaced and unspaced formats)
export const parseServiceId = (command: string): number | null => {
  const trimmed = command.trim().toUpperCase();
  if (!trimmed) return null;

  // Spaced format: "2E F1 90"
  const parts = trimmed.split(/\s+/);
  if (parts[0].length === 2) {
    const val = parseInt(parts[0], 16);
    return isNaN(val) ? null : val;
  }

  // Unspaced format: "2EF190"
  if (trimmed.length >= 2) {
    const val = parseInt(trimmed.substring(0, 2), 16);
    return isNaN(val) ? null : val;
  }

  return null;
};

/// Match the backend's default-deny raw-command policy. The console is a
/// diagnostic reader, not a generic ECU programming surface.
export const isCommandBlocked = (command: string): boolean => {
  const trimmed = command.trim().toUpperCase();
  if (!trimmed) return false;

  // The backend normalizes raw commands as hexadecimal, so AT commands are
  // intentionally unavailable from this screen.
  if (trimmed.startsWith("AT")) return true;

  const serviceId = parseServiceId(command);
  if (serviceId === null) return true;
  const readOnlyServices = [
    0x01, 0x02, 0x03, 0x05, 0x06, 0x07, 0x09, 0x0a,
    0x19, 0x22, 0x3e,
  ];
  return !readOnlyServices.includes(serviceId);
};

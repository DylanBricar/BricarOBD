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

/// Check if command is blocked (matches ALWAYS_BLOCKED list from backend)
export const isCommandBlocked = (command: string): boolean => {
  const trimmed = command.trim().toUpperCase();

  // Blocked AT commands
  const blockedAt = ["ATMA", "ATBD", "ATBI", "ATPP", "ATWS"];
  for (const at of blockedAt) {
    if (trimmed.startsWith(at)) {
      return true;
    }
  }

  // Blocked service IDs (always blocked even in advanced mode)
  const alwaysBlocked = [0x11, 0x27, 0x28, 0x34, 0x35, 0x36, 0x37, 0x3D];
  const serviceId = parseServiceId(command);
  if (serviceId !== null && alwaysBlocked.includes(serviceId)) {
    return true;
  }

  return false;
};

export function hasModifierToken(hotkey: string): boolean {
  const modifierTokens = new Set([
    'Cmd',
    'Command',
    'Control',
    'Ctrl',
    'Option',
    'Alt',
    'Shift',
    'Meta',
    'Super'
  ]);
  return hotkey
    .split('+')
    .map((token) => token.trim())
    .some((token) => modifierTokens.has(token));
}

export function isFunctionKeyToken(token: string): boolean {
  return /^F([1-9]|1[0-9]|2[0-4])$/i.test(token.trim());
}

export function allowsHotkeyWithoutModifier(hotkey: string): boolean {
  const tokens = hotkey
    .split('+')
    .map((token) => token.trim())
    .filter((token) => token.length > 0);
  return tokens.length === 1 && isFunctionKeyToken(tokens[0]);
}

export function isModifierOnlyKey(key: string): boolean {
  return key === 'Meta' || key === 'Control' || key === 'Shift' || key === 'Alt';
}

export function normalizeMainHotkeyKey(event: KeyboardEvent): string | null {
  if (isModifierOnlyKey(event.key)) {
    return null;
  }

  if (/^[a-z]$/i.test(event.key)) {
    return event.key.toUpperCase();
  }

  if (/^[0-9]$/.test(event.key)) {
    return event.key;
  }

  if (/^F([1-9]|1[0-9]|2[0-4])$/i.test(event.key)) {
    return event.key.toUpperCase();
  }

  switch (event.key) {
    case 'Space':
    case ' ':
    case 'Spacebar':
      return 'Space';
    case 'ArrowUp':
      return 'Up';
    case 'ArrowDown':
      return 'Down';
    case 'ArrowLeft':
      return 'Left';
    case 'ArrowRight':
      return 'Right';
    case 'Enter':
      return 'Enter';
    case 'Tab':
      return 'Tab';
    case 'Backspace':
      return 'Backspace';
    case 'Delete':
      return 'Delete';
    case 'Home':
      return 'Home';
    case 'End':
      return 'End';
    case 'PageUp':
      return 'PageUp';
    case 'PageDown':
      return 'PageDown';
    case 'Escape':
      return 'Escape';
    default:
      break;
  }

  if (event.code.startsWith('Key') && event.code.length === 4) {
    return event.code.slice(3).toUpperCase();
  }

  if (event.code.startsWith('Digit') && event.code.length === 6) {
    return event.code.slice(5);
  }

  const codeMap: Record<string, string> = {
    Backquote: 'Backquote',
    Minus: 'Minus',
    Equal: 'Equal',
    BracketLeft: 'BracketLeft',
    BracketRight: 'BracketRight',
    Backslash: 'Backslash',
    Semicolon: 'Semicolon',
    Quote: 'Quote',
    Comma: 'Comma',
    Period: 'Period',
    Slash: 'Slash',
    Insert: 'Insert',
    PrintScreen: 'PrintScreen',
    Numpad0: 'Numpad0',
    Numpad1: 'Numpad1',
    Numpad2: 'Numpad2',
    Numpad3: 'Numpad3',
    Numpad4: 'Numpad4',
    Numpad5: 'Numpad5',
    Numpad6: 'Numpad6',
    Numpad7: 'Numpad7',
    Numpad8: 'Numpad8',
    Numpad9: 'Numpad9',
    NumpadAdd: 'NumpadAdd',
    NumpadSubtract: 'NumpadSubtract',
    NumpadMultiply: 'NumpadMultiply',
    NumpadDivide: 'NumpadDivide',
    NumpadDecimal: 'NumpadDecimal',
    NumpadEnter: 'NumpadEnter'
  };
  if (codeMap[event.code]) {
    return codeMap[event.code];
  }

  return null;
}

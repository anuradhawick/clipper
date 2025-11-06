export const tauriHotkeyToBrowserHotkey = (
  hotkeyStr: string
): {
  ctrl: boolean;
  alt: boolean;
  shift: boolean;
  meta: boolean;
  key: string;
} => {
  // Normalize and split, ignore whitespace
  const tokens = hotkeyStr.split("+").map((t) => t.trim().toLowerCase());

  // Modifiers
  let ctrl = false,
    alt = false,
    shift = false,
    meta = false;
  let key = "";

  // Helper: key code mapping (as before, all lower/unshifted)
  const codeMap: Record<string, string> = {
    backquote: "`",
    backslash: "\\",
    bracketleft: "[",
    bracketright: "]",
    comma: ",",
    digit0: "0",
    digit1: "1",
    digit2: "2",
    digit3: "3",
    digit4: "4",
    digit5: "5",
    digit6: "6",
    digit7: "7",
    digit8: "8",
    digit9: "9",
    equal: "=",
    keya: "a",
    keyb: "b",
    keyc: "c",
    keyd: "d",
    keye: "e",
    keyf: "f",
    keyg: "g",
    keyh: "h",
    keyi: "i",
    keyj: "j",
    keyk: "k",
    keyl: "l",
    keym: "m",
    keyn: "n",
    keyo: "o",
    keyp: "p",
    keyq: "q",
    keyr: "r",
    keys: "s",
    keyt: "t",
    keyu: "u",
    keyv: "v",
    keyw: "w",
    keyx: "x",
    keyy: "y",
    keyz: "z",
    minus: "-",
    period: ".",
    quote: "'",
    semicolon: ";",
    slash: "/",
    space: " ",
    tab: "Tab",
    enter: "Enter",
    backspace: "Backspace",
    escape: "Escape",
    capslock: "CapsLock",
    delete: "Delete",
    end: "End",
    home: "Home",
    insert: "Insert",
    pagedown: "PageDown",
    pageup: "PageUp",
    printscreen: "PrintScreen",
    scrolllock: "ScrollLock",
    arrowdown: "ArrowDown",
    arrowleft: "ArrowLeft",
    arrowright: "ArrowRight",
    arrowup: "ArrowUp",
    numlock: "NumLock",
    numpad0: "0",
    numpad1: "1",
    numpad2: "2",
    numpad3: "3",
    numpad4: "4",
    numpad5: "5",
    numpad6: "6",
    numpad7: "7",
    numpad8: "8",
    numpad9: "9",
    numpadadd: "+",
    numpaddecimal: ".",
    numpaddivide: "/",
    numpadenter: "Enter",
    numpadequal: "=",
    numpadmultiply: "*",
    numpadsubtract: "-",
    f1: "F1",
    f2: "F2",
    f3: "F3",
    f4: "F4",
    f5: "F5",
    f6: "F6",
    f7: "F7",
    f8: "F8",
    f9: "F9",
    f10: "F10",
    f11: "F11",
    f12: "F12",
    f13: "F13",
    f14: "F14",
    f15: "F15",
    f16: "F16",
    f17: "F17",
    f18: "F18",
    f19: "F19",
    f20: "F20",
    f21: "F21",
    f22: "F22",
    f23: "F23",
    f24: "F24",
    audiovolumedown: "AudioVolumeDown",
    audiovolumeup: "AudioVolumeUp",
    audiovolumemute: "AudioVolumeMute",
    mediaplay: "MediaPlay",
    mediapause: "MediaPause",
    mediaplaypause: "MediaPlayPause",
    mediastop: "MediaStop",
    mediatracknext: "MediaTrackNext",
    mediatrackprevious: "MediaTrackPrevious",
    // add more as needed
  };

  // Tauri supports these modifiers (case-insensitive)
  for (const t of tokens) {
    if (["ctrl", "control"].includes(t)) ctrl = true;
    else if (["alt", "option"].includes(t)) alt = true;
    else if (["shift"].includes(t)) shift = true;
    else if (["super", "cmd", "command", "meta"].includes(t)) meta = true;
    else if (["cmdorctrl", "commandorcontrol"].includes(t)) {
      // On macOS, it's usually meta; elsewhere it's ctrl. Most apps treat it as ctrl on browser.
      ctrl = true;
    } else {
      // If not a modifier, it's the key code (e.g. KeyQ, Digit2, F1...)
      // Map to browser key value
      key = codeMap[t] || t;
    }
  }

  return { ctrl, alt, shift, meta, key };
};

export const browserHotkeyToMacSymbols = (hotkey: {
  ctrl: boolean;
  alt: boolean;
  shift: boolean;
  meta: boolean;
  key: string;
}) => {
  // macOS modifier symbols in standard display order
  const parts = [];
  if (hotkey.meta) parts.push("⌘");
  if (hotkey.shift) parts.push("⇧");
  if (hotkey.alt) parts.push("⌥");
  if (hotkey.ctrl) parts.push("⌃");

  // Map special keys
  const specialKeys: Record<string, string> = {
    enter: "↩",
    return: "↩",
    tab: "⇥",
    backspace: "⌫",
    delete: "⌦",
    escape: "⎋",
    esc: "⎋",
    arrowleft: "←",
    arrowup: "↑",
    arrowright: "→",
    arrowdown: "↓",
    space: "␣",
    capslock: "⇪",
    pageup: "⇞",
    pagedown: "⇟",
    home: "↖",
    end: "↘",
    f1: "F1",
    f2: "F2",
    f3: "F3",
    f4: "F4",
    f5: "F5",
    f6: "F6",
    f7: "F7",
    f8: "F8",
    f9: "F9",
    f10: "F10",
    f11: "F11",
    f12: "F12",
    f13: "F13",
    f14: "F14",
    f15: "F15",
    f16: "F16",
    f17: "F17",
    f18: "F18",
    f19: "F19",
    f20: "F20",
  };

  let key = hotkey.key;

  // Try special key mapping, otherwise use uppercase for a-z/0-9
  let displayKey = specialKeys[key.toLowerCase()];
  if (!displayKey && typeof key === "string") {
    // For printable characters, show uppercase
    if (/^[a-z0-9]$/.test(key)) displayKey = key.toUpperCase();
    // For others (punctuation), show as-is
    else displayKey = key;
  }

  if (displayKey) parts.push(displayKey);

  // Join with nothing (macOS shows symbols together)
  return parts.join(" + ");
};

export const browserHotkeyToLinuxString = (hotkey: {
  ctrl: boolean;
  alt: boolean;
  shift: boolean;
  meta: boolean;
  key: string;
}) => {
  const parts = [];
  if (hotkey.ctrl) parts.push("Ctrl");
  if (hotkey.shift) parts.push("Shift");
  if (hotkey.alt) parts.push("Alt");
  if (hotkey.meta) parts.push("Super"); // Or "Meta" or "Win"

  // Map special keys
  const specialKeys: Record<string, string> = {
    enter: "Enter",
    return: "Enter",
    tab: "Tab",
    backspace: "Backspace",
    delete: "Delete",
    escape: "Esc",
    esc: "Esc",
    arrowleft: "Left",
    arrowup: "Up",
    arrowright: "Right",
    arrowdown: "Down",
    space: "Space",
    capslock: "CapsLock",
    pageup: "PageUp",
    pagedown: "PageDown",
    home: "Home",
    end: "End",
    f1: "F1",
    f2: "F2",
    f3: "F3",
    f4: "F4",
    f5: "F5",
    f6: "F6",
    f7: "F7",
    f8: "F8",
    f9: "F9",
    f10: "F10",
    f11: "F11",
    f12: "F12",
    f13: "F13",
    f14: "F14",
    f15: "F15",
    f16: "F16",
    f17: "F17",
    f18: "F18",
    f19: "F19",
    f20: "F20",
  };

  let key = hotkey.key;

  // Try special key mapping, otherwise use uppercase for a-z/0-9
  let displayKey = specialKeys[key.toLowerCase()];
  if (!displayKey && typeof key === "string") {
    // For printable characters, show uppercase for a-z/0-9
    if (/^[a-z0-9]$/.test(key)) displayKey = key.toUpperCase();
    // For others (punctuation), show as-is
    else displayKey = key;
  }

  if (displayKey) parts.push(displayKey);

  return parts.join(" + ");
};

export const isValidHotkey = (codes: Set<string>): boolean => {
  // Modifier code prefixes
  const modifierPrefixes = ["Control", "Shift", "Alt", "Meta"];

  let hasModifier = false;
  let hasNonModifier = false;

  for (const code of codes) {
    // If code starts with any modifier prefix, it's a modifier
    if (modifierPrefixes.some((prefix) => code.startsWith(prefix))) {
      hasModifier = true;
    } else {
      hasNonModifier = true;
    }
    // Early exit if both found
    if (hasModifier && hasNonModifier) return true;
  }

  return false;
};

export const browserKeyCodesToTauriHotkey = (codes: Set<string>): string => {
  // Modifier code prefixes for browser KeyboardEvent.code
  const modifiersOrder = ["Control", "Shift", "Alt", "Meta"];
  const modifierMap: Record<string, string> = {
    Control: "ctrl",
    Shift: "shift",
    Alt: "alt",
    Meta: "super",
  };

  // Collect modifiers and find the main key code
  const modifiers: string[] = [];
  let nonModifierKey: string | undefined;

  for (const mod of modifiersOrder) {
    // Detect modifier codes (e.g., ControlLeft, ControlRight)
    const hasModifier = Array.from(codes).some((code) => code.startsWith(mod));
    if (hasModifier) {
      modifiers.push(modifierMap[mod]);
    }
  }

  // Find the non-modifier key
  for (const code of codes) {
    // Not a modifier if it doesn't start with a modifier prefix
    if (!modifiersOrder.some((prefix) => code.startsWith(prefix))) {
      nonModifierKey = code;
      break;
    }
  }

  if (!nonModifierKey) {
    throw new Error("No non-modifier key found!");
  }

  // For most codes (KeyA, Digit1, F1, ArrowLeft, etc.), we can use directly.
  // However, we should map some legacy or less common browser codes to Tauri equivalents if needed.
  const specialKeyMap: Record<string, string> = {
    Space: "Space",
    Spacebar: "Space",
    Backspace: "Backspace",
    Enter: "Enter",
    Tab: "Tab",
    Escape: "Escape",
    Esc: "Escape",
    ArrowLeft: "ArrowLeft",
    ArrowRight: "ArrowRight",
    ArrowUp: "ArrowUp",
    ArrowDown: "ArrowDown",
    CapsLock: "CapsLock",
    Delete: "Delete",
    End: "End",
    Home: "Home",
    Insert: "Insert",
    PageDown: "PageDown",
    PageUp: "PageUp",
    PrintScreen: "PrintScreen",
    ScrollLock: "ScrollLock",
    NumLock: "NumLock",
    Numpad0: "Numpad0",
    Numpad1: "Numpad1",
    Numpad2: "Numpad2",
    Numpad3: "Numpad3",
    Numpad4: "Numpad4",
    Numpad5: "Numpad5",
    Numpad6: "Numpad6",
    Numpad7: "Numpad7",
    Numpad8: "Numpad8",
    Numpad9: "Numpad9",
    NumpadAdd: "NumpadAdd",
    NumpadDecimal: "NumpadDecimal",
    NumpadDivide: "NumpadDivide",
    NumpadEnter: "NumpadEnter",
    NumpadEqual: "NumpadEqual",
    NumpadMultiply: "NumpadMultiply",
    NumpadSubtract: "NumpadSubtract",
    F1: "F1",
    F2: "F2",
    F3: "F3",
    F4: "F4",
    F5: "F5",
    F6: "F6",
    F7: "F7",
    F8: "F8",
    F9: "F9",
    F10: "F10",
    F11: "F11",
    F12: "F12",
    // Extend as needed
  };

  // Use the mapping if present, otherwise use the code as-is
  const tauriKey = specialKeyMap[nonModifierKey] || nonModifierKey;

  return [...modifiers, tauriKey].join("+");
};

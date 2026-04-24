export const processBytes = (
  bytes: Array<number>,
): { content: string; type: string }[] => {
  let text = new TextDecoder().decode(Uint8Array.from(bytes));
  const urlRegex = /(https?:\/\/[^\s]+)/g;
  const parts = text.split(urlRegex);

  const transformedParts = parts.map((part) => {
    if (urlRegex.test(part)) {
      return {
        content: part,
        type: "url",
      };
    }
    return { content: part, type: "text" };
  });
  return transformedParts;
};

export const asPlainText = (bytes: Array<number>): string => {
  return new TextDecoder().decode(Uint8Array.from(bytes));
};

export const processText = (
  text: string,
): { content: string; type: string }[] => {
  const urlRegex = /(https?:\/\/[^\s]+)/g;
  const parts = text.split(urlRegex);

  const transformedParts = parts.map((part) => {
    if (urlRegex.test(part)) {
      return {
        content: part,
        type: "url",
      };
    }
    return { content: part, type: "text" };
  });
  return transformedParts;
};

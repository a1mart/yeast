// Utility function to format timestamp to readable date
export const formatDate = (timestamp) => {
  const date = new Date(timestamp * 1000);
  return date.toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    year:
      timestamp < Date.now() / 1000 - 365 * 24 * 60 * 60
        ? 'numeric'
        : undefined,
  });
};

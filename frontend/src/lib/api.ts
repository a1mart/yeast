// API configuration - replace with your actual API base URL
const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || '/api';

export const api = {
  get: async (endpoint, options = {}) => {
    const url = new URL(`${API_BASE_URL}${endpoint}`, window.location.origin);

    if (options.params) {
      Object.keys(options.params).forEach((key) => {
        if (options.params[key] !== undefined && options.params[key] !== null) {
          url.searchParams.append(key, options.params[key]);
        }
      });
    }

    const response = await fetch(url.toString(), {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
    });

    if (!response.ok) {
      throw new Error(`API Error: ${response.status} ${response.statusText}`);
    }

    return await response.json();
  },

  post: async (endpoint, data, options = {}) => {
    const response = await fetch(`${API_BASE_URL}${endpoint}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
      body: JSON.stringify(data),
    });

    if (!response.ok) {
      throw new Error(`API Error: ${response.status} ${response.statusText}`);
    }

    return await response.json();
  },
};

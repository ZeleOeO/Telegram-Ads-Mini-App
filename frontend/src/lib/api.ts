import axios from 'axios';

// Create axios instance
export const api = axios.create({
    baseURL: '', // Relative path since we serve from same origin
    headers: {
        'Content-Type': 'application/json',

        'Bypass-Tunnel-Reminder': 'true',
    },
});

// Helper to set auth token dynamically
export const setAuthToken = (token: string) => {
    if (token) {
        // Clean the token just in case
        const cleanToken = token.trim().replace(/[\r\n]/g, '');
        api.defaults.headers.common['Authorization'] = `tma ${cleanToken}`;
    } else {
        delete api.defaults.headers.common['Authorization'];
    }
};

api.interceptors.response.use(
    (response) => response,
    (error) => {
        return Promise.reject(error);
    }
);

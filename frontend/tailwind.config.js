/** @type {import('tailwindcss').Config} */
export default {
    content: [
        "./index.html",
        "./src/**/*.{js,ts,jsx,tsx}",
    ],
    theme: {
        extend: {
            colors: {
                background: '#0b0e14',
                surface: '#151a23',
                primary: '#3b82f6',
                secondary: '#64748b',
                accent: '#10b981',
                danger: '#ef4444',
            },
            fontFamily: {
                sans: ['Outfit', 'sans-serif'],
            },
        },
    },
    plugins: [],
}

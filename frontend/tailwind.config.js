/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        primary: {
          bg: '#F5F7F6',
          white: '#FFFFFF',
          sidebar: '#F8F9FA',
        },
        text: {
          primary: '#212529',
          secondary: '#6C757D',
          muted: '#979A98',
        },
        accent: {
          green: '#10B981',
          greenDark: '#059669',
          darkGreen: '#107130',
          sage: '#AECDB1',
        },
        danger: {
          red: '#DC2626',
        },
        border: {
          DEFAULT: '#E9ECEF',
          input: '#CED4DA',
        },
        dark: {
          bg: '#131313',
          surface: '#1E1E1E',
          border: '#333333',
          text: '#FFFFFF',
          textSecondary: '#979A98',
        },
      },
      fontFamily: {
        sans: ['Inter', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'sans-serif'],
      },
      spacing: {
        '1': '4px',
        '2': '8px',
        '3': '12px',
        '4': '16px',
        '5': '20px',
        '6': '24px',
        '8': '32px',
        '10': '40px',
        '12': '48px',
      },
      borderRadius: {
        'sm': '4px',
        'md': '8px',
        'lg': '12px',
        'full': '9999px',
      },
      boxShadow: {
        'sm': '0 1px 2px rgba(0, 0, 0, 0.05)',
        'md': '0 4px 6px -1px rgba(0, 0, 0, 0.1)',
        'lg': '0 10px 15px -3px rgba(0, 0, 0, 0.1)',
      },
    },
  },
  plugins: [],
}

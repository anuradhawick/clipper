/** @type {import('tailwindcss').Config} */
module.exports = {
  corePlugins: {
    preflight: true,
  },
  darkMode: "selector",
  content: ["./src/**/*.{html,ts,scss}"],
  theme: {
    extend: {},
  },
  plugins: [],
};

/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./src/**/*.{html,js,ts,svelte}",
  ],
  theme: {
    extend: {},
  },
  plugins: [
    require("daisyui"),
  ],
  daisyui: {
    themes: ["dim"],
    darkTheme: "dim",
    base: true,
    styled: true,
    utils: true,
    logs: false,
  },
};

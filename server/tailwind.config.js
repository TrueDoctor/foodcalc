export const content = ["./src/frontend/**/*.rs", "./src/htmx_middleware.rs", "./../foodlib_new/src/error.rs"]

import { createRequire } from 'module';
const require = createRequire(import.meta.url);

const colors = require('tailwindcss/colors')

export const theme = {
  extend: {
    colors: {
      light: {
        primary: {
          normal: colors.blue[600],
          hover: colors.blue[700],
          light: colors.blue[400],
        },
        secondary: {
          normal: "#dddbd8",
          hover: "#c8c6c4",
          light: "#b0adab",
        },
        highlight: {
          normal: "#3f5ab1",
          hover: "#2c4ca1",
          highlight: "#1a3e91",
        },
        bg: {
          dark: colors.slate[200],
          light: colors.slate[100],
        },
      },
      dark: {
        primary: {
          normal: colors.blue[600],
          hover: colors.blue[700],
          light: colors.blue[400],
        },
        secondary: {
          normal: "#272727",
          hover: "#3e3e3e",
          highlight: "#555555",
        },
        highlight: {
          normal: "#ffc857",
          hover: "#ffbf69",
          highlight: "#ffad00",
        },
        bg: {
          dark: colors.gray[800],
          light: colors.gray[900],
        }
      },
      navbar: colors.blue[700],
      btn: {
        success: {
          normal: colors.green[600],
          hover: colors.green[700],
        },
        cancel: {
          normal: colors.red[600],
          hover: colors.red[700],
        },
      },
    },
  },
  plugins: [
    require('@tailwindcss/forms'),
  ],
}

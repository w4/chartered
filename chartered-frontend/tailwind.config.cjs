/** @type {import('tailwindcss').Config} */
module.exports = {
    darkMode: 'media',
    content: ['./src/**/*.{html,js,svelte,ts}'],
    theme: {
        screens: {
            sm: '640px',
            md: '800px',
            lg: '1024px',
            xl: '1280px',
            '2xl': '1536px',
        },
        extend: {
            gridTemplateColumns: {
                'sides-extend': 'minmax(20px, 1fr) auto minmax(20px, 1fr)',
            },
        },
    },
    plugins: [require('@tailwindcss/forms'), require('@tailwindcss/typography')],
};

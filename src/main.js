import { createApp } from "vue";
import { createI18n } from 'vue-i18n'
import createRouter from './router/index'
import messages from './i18n/index'

import "./styles.css";
import App from "./App.vue";
import { Quasar } from 'quasar'
import quasarUserOptions from './quasar-user-options'

// Import icon libraries
import '@quasar/extras/material-icons/material-icons.css'

// Import Quasar css
import 'quasar/src/css/index.sass'


const app = createApp(App)
app.use(Quasar, quasarUserOptions)

console.log(`main.js start app and use quasar, i18n, router and mound #app`)
const i18n = createI18n({
    legacy: true,
    locale: 'de-DE',         // set locale
    fallbackLocale: 'en-US', // set fallback locale
    messages,                // set locale messages
    //datetimeFormats,
    numberFormats: {
        'de-DE': {
            currency: {
                style: 'currency',
                currency: 'EUR'
            }
        },
        'en-US': {
            currency: {
                style: 'currency',
                currency: 'USD'
            }
        }
    }
})

app.use(i18n)

const router = createRouter();

app.use(router)

app.mount("#app");

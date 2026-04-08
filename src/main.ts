import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import './styles.css';
import './modals.css';

createApp(App).use(createPinia()).mount('#app');

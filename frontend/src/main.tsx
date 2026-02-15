import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { BrowserRouter } from 'react-router-dom'
import { TonConnectUIProvider } from '@tonconnect/ui-react';
import './index.css'
import App from './App.tsx'

// Ensure Telegram WebApp is ready
import WebApp from '@twa-dev/sdk'
WebApp.ready()

const manifestUrl = new URL('/tonconnect-manifest.json', window.location.href).toString();
// const manifestUrl = "https://gist.githubusercontent.com/ZeleOeO/6b41e08b813f0be86cb01046943983e5/raw/bb238076f41bdd3c739254dab5e8f21d2ca804bb/tonconnect-manifest.json";

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <TonConnectUIProvider manifestUrl={manifestUrl}>
      <BrowserRouter>
        <App />
      </BrowserRouter>
    </TonConnectUIProvider>
  </StrictMode>,
)

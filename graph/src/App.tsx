import React from 'react';
import 'bootstrap/dist/css/bootstrap.min.css';
import 'bootstrap/dist/js/bootstrap.min.js'
import './App.css';
import { ClientLayout } from './Layout';
import Home from './Home';
import { ToastProvider } from './utils/Toast';

function App() {
  return (
    <ToastProvider>
      <ClientLayout>
        <Home />
      </ClientLayout>
    </ToastProvider>
  );
}

export default App;

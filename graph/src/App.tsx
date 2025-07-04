import React from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import 'bootstrap/dist/css/bootstrap.min.css';
import 'bootstrap/dist/js/bootstrap.min.js'

import './App.css';
import { ClientLayout } from './Layout';
import Home from './Home';
import { ToastProvider } from './utils/Toast';
import { Ledger } from './Ledger';
import { DailyBalances } from './DailyBalances';
function App() {
  return (
    <ToastProvider>
       <BrowserRouter>
        <ClientLayout>
          <Routes>
            <Route index path="/" element={<Home />} />
            <Route path="/ledger" element={<Ledger />} />
            <Route path="/balances" element={<DailyBalances />} />
          </Routes>
        </ClientLayout>
      </BrowserRouter>
    </ToastProvider>
  );
}

export default App;

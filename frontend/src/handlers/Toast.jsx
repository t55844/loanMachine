// components/Toast.jsx
import { useState, useEffect } from 'react';
import { eventSystem } from '../handlers/EventSystem';
import { extractErrorMessage, getErrorType } from '../handlers/errorMapping';

export default function Toast() {
  const [toast, setToast] = useState({ show: false, message: '', type: 'info' });

  useEffect(() => {
    // Listen for toast events
    eventSystem.on('showToast', (data) => {
      // Use error mapping if it's an error object
      const message = data.isError ? extractErrorMessage(data.message) : data.message;
      const type = data.isError ? getErrorType(data.message) : data.type;
      
      setToast({ 
        show: true, 
        message, 
        type: type || 'info'
      });
      
      // Auto hide after duration or default 5 seconds
      setTimeout(() => {
        setToast(prev => ({ ...prev, show: false }));
      }, data.duration || 5000);
    });
  }, []);

  if (!toast.show) return null;

  const bgColor = {
    success: '#10b981',
    error: '#ef4444', 
    warning: '#f59e0b',
    info: '#3b82f6'
  }[toast.type];

  return (
    <div style={{
      position: 'fixed',
      top: '20px',
      right: '20px',
      background: bgColor,
      color: 'white',
      padding: '16px',
      borderRadius: '8px',
      boxShadow: '0 4px 12px rgba(0,0,0,0.3)',
      zIndex: 10000,
      maxWidth: '400px'
    }}>
      {toast.message}
      <button 
        onClick={() => setToast(prev => ({ ...prev, show: false }))}
        style={{
          background: 'none',
          border: 'none',
          color: 'white',
          marginLeft: '10px',
          cursor: 'pointer'
        }}
      >
        ×
      </button>
    </div>
  );
}
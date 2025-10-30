// components/GlobalErrorHandler.jsx
import { useEffect, useState } from 'react';
import { eventBus, EVENT_TYPES } from '../handlers/EventSystem';
import EnhancedToast from './EnhancedToast';

export default function GlobalErrorHandler() {
  const [toasts, setToasts] = useState([]);

  useEffect(() => {
    // Subscribe to show toast events
    const unsubscribeShow = eventBus.subscribe(
      EVENT_TYPES.SHOW_TOAST,
      (toastData) => {
        const toastId = Date.now() + Math.random();
        const newToast = {
          id: toastId,
          ...toastData,
          show: true
        };

        setToasts(prev => [...prev, newToast]);

        // Auto-remove if duration is set
        if (toastData.duration && toastData.duration > 0) {
          setTimeout(() => {
            removeToast(toastId);
          }, toastData.duration);
        }
      }
    );

    // Subscribe to hide toast events
    const unsubscribeHide = eventBus.subscribe(
      EVENT_TYPES.HIDE_TOAST,
      (toastId) => {
        removeToast(toastId);
      }
    );

    // Subscribe to global transaction events for analytics/logging
    const unsubscribeTxStarted = eventBus.subscribe(
      EVENT_TYPES.TRANSACTION_STARTED,
      (data) => {
        console.log('Transaction started:', data);
        // You can add analytics tracking here
      }
    );

    const unsubscribeTxSuccess = eventBus.subscribe(
      EVENT_TYPES.TRANSACTION_SUCCESS,
      (data) => {
        console.log('Transaction succeeded:', data);
        // Analytics tracking for successful transactions
      }
    );

    const unsubscribeTxError = eventBus.subscribe(
      EVENT_TYPES.TRANSACTION_ERROR,
      (data) => {
        console.log('Transaction failed:', data);
        // Error logging service integration
      }
    );

    return () => {
      unsubscribeShow();
      unsubscribeHide();
      unsubscribeTxStarted();
      unsubscribeTxSuccess();
      unsubscribeTxError();
    };
  }, []);

  const removeToast = (id) => {
    setToasts(prev => prev.map(toast => 
      toast.id === id ? { ...toast, show: false } : toast
    ));
    
    setTimeout(() => {
      setToasts(prev => prev.filter(toast => toast.id !== id));
    }, 300);
  };

  return (
    <div style={{
      position: 'fixed',
      top: '20px',
      right: '20px',
      zIndex: 10000,
      display: 'flex',
      flexDirection: 'column',
      gap: '10px'
    }}>
      {toasts.map((toast, index) => (
        <div 
          key={toast.id}
          style={{
            transform: `translateY(${index * 10}px)`
          }}
        >
          <EnhancedToast 
            toast={toast} 
            onClose={() => removeToast(toast.id)} 
          />
        </div>
      ))}
    </div>
  );
}
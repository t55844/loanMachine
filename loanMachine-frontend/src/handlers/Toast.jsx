// components/Toast.jsx
export default function Toast({ toast, onClose }) {
  if (!toast.show) return null;

  const bgColor = toast.type === 'success' 
    ? 'var(--accent-green)' 
    : toast.type === 'warning' 
    ? '#eab308' 
    : 'var(--accent-red)';

  return (
    <div style={{
      position: 'fixed',
      top: '20px',
      right: '20px',
      background: bgColor,
      color: 'white',
      padding: '16px',
      borderRadius: '8px',
      boxShadow: '0 4px 12px rgba(0,0,0,0.4)',
      zIndex: 10000,
      maxWidth: '400px',
      animation: 'slideIn 0.3s ease'
    }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', gap: '12px' }}>
        <span>{toast.message}</span>
        <button 
          onClick={onClose}
          style={{
            background: 'none',
            border: 'none',
            color: 'white',
            fontSize: '18px',
            cursor: 'pointer',
            padding: 0,
            width: '20px',
            height: '20px'
          }}
        >
          ×
        </button>
      </div>
      
      <style jsx>{`
        @keyframes slideIn {
          from { transform: translateX(100%); }
          to { transform: translateX(0); }
        }
      `}</style>
    </div>
  );
}
import { useState, useRef, useEffect } from "react";

export default function SideMenu({ children, position = "left" }) {
  const [isOpen, setIsOpen] = useState(false);
  const [isVisible, setIsVisible] = useState(false);
  const menuRef = useRef(null);

  const toggleMenu = () => {
    if (!isOpen) {
      setIsOpen(true);
      // Small delay to ensure the element is rendered before starting animation
      setTimeout(() => setIsVisible(true), 10);
    } else {
      setIsVisible(false);
      // Wait for animation to complete before hiding the element
      setTimeout(() => setIsOpen(false), 300);
    }
  };

  // Close menu when clicking outside
  useEffect(() => {
    const handleClickOutside = (event) => {
      if (menuRef.current && !menuRef.current.contains(event.target)) {
        if (isOpen) {
          setIsVisible(false);
          setTimeout(() => setIsOpen(false), 300);
        }
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [isOpen]);

  return (
    <>
      {/* Toggle Button */}
      <button 
        className={`menu-toggle ${position} ${isOpen ? 'open' : ''}`}
        onClick={toggleMenu}
      >
        <span></span>
        <span></span>
        <span></span>
      </button>

      {/* Overlay */}
      {isOpen && (
        <div 
          className={`menu-overlay ${isVisible ? 'visible' : ''}`}
          onClick={toggleMenu}
        ></div>
      )}

      {/* Side Menu */}
      {isOpen && (
        <div 
          ref={menuRef}
          className={`side-menu ${position} ${isVisible ? 'visible' : ''}`}
        >
          <div className="menu-header">
            <h3>Painel do Usuário</h3>
            <button className="close-button" onClick={toggleMenu}>
              ×
            </button>
          </div>
          <div className="menu-content">
            {children}
          </div>
        </div>
      )}
    </>
  );
}
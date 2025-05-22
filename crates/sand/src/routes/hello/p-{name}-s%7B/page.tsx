import {useEffect, useMemo, useState} from 'react'
import {useServerValues} from 'ruxyjs/server'

// noinspection JSUnusedGlobalSymbols
export default function Page() {
  return (
    <div>
      <MessageBar />
      <StyledClientContent />
    </div>
  );
}

function MessageBar() {
  const { message } = useServerValues();
  const enhancedMessage = useTogglingCasingWithSuffix(message);
  
  if (!enhancedMessage) return (
    <div>No message received from server.</div>
  );
  
  return (
    <div className="container">
      {enhancedMessage}
    </div>
  );
}

/**
 * Hook that appends a suffix and switches casing every second
 * */
function useTogglingCasingWithSuffix(text?: string) {
  const [current, setCurrent] = useState({ text, casing: 'original' });
  
  useEffect(() => {
    if (!current.text) return;
    
    const interval = window.setInterval(() => {
      setCurrent(prev => {
        const newCasing = prev.casing === 'upper' ? 'lower' : 'upper';
        
        return {
          text: newCasing === 'upper'
            ? text.toUpperCase()
            : text.toLowerCase(),
          casing: newCasing,
        }
      });
    }, 1_000);
    
    return () => {
      window.clearInterval(interval);
    };
  }, [current])
  
  const message = (current ?? 'No message') + '!';
  
  return (
    <>
      <div>{message}</div>
      {current.casing === 'upper' && <div>(uppercased)</div>}
      {current.casing === 'lower' && <div>(lowercased)</div>}
    </>
  )
}

function StyledClientContent() {
  const [shown, setShown] = useState(false);
  
  const onClick = useMemo(() => () => setShown(true), []);
  
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
      <button
        style={{ display: 'block' }}
        onClick={onClick}
      >
        Show client content
      </button>
      
      {shown && <span>Client content</span>}
    </div>
  );
}

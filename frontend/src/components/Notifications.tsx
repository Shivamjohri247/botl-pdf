import { useState, useEffect } from 'react'
import { Bell, X, CheckCircle, XCircle, AlertCircle, Info } from 'lucide-react'
import { useNotifications } from '../contexts/NotificationContext'

export function NotificationBell() {
  const { notifications, unreadCount, markAllAsRead, clearAll } = useNotifications()
  const [isOpen, setIsOpen] = useState(false)

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = () => {
      if (isOpen) setIsOpen(false)
    }
    document.addEventListener('click', handleClickOutside)
    return () => document.removeEventListener('click', handleClickOutside)
  }, [isOpen])

  const handleToggle = (e: React.MouseEvent) => {
    e.stopPropagation()
    setIsOpen(!isOpen)
    if (!isOpen) markAllAsRead()
  }

  const getIcon = (type: string) => {
    switch (type) {
      case 'success': return <CheckCircle className="w-4 h-4 text-emerald-500" />
      case 'error': return <XCircle className="w-4 h-4 text-red-500" />
      case 'warning': return <AlertCircle className="w-4 h-4 text-amber-500" />
      default: return <Info className="w-4 h-4 text-blue-500" />
    }
  }

  return (
    <div className="relative">
      <button
        onClick={handleToggle}
        className="relative p-2 text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded-lg transition-colors"
        title="Notifications"
      >
        <Bell className="w-5 h-5" />
        {unreadCount > 0 && (
          <span className="absolute top-1 right-1 w-5 h-5 bg-red-500 text-white text-xs rounded-full flex items-center justify-center">
            {unreadCount > 9 ? '9+' : unreadCount}
          </span>
        )}
      </button>

      {isOpen && (
        <div className="absolute right-0 top-full mt-2 w-80 bg-white rounded-lg shadow-xl border border-gray-200 z-50">
          <div className="p-4 border-b border-gray-200 flex items-center justify-between">
            <h3 className="font-semibold text-gray-900">Notifications</h3>
            <div className="flex gap-2">
              <button
                onClick={(e) => { e.stopPropagation(); clearAll() }}
                className="text-xs text-gray-500 hover:text-gray-700"
              >
                Clear All
              </button>
            </div>
          </div>

          <div className="max-h-80 overflow-y-auto">
            {notifications.length === 0 ? (
              <div className="p-8 text-center text-gray-500">
                <Bell className="w-8 h-8 mx-auto mb-2 text-gray-400" />
                <p className="text-sm">No notifications</p>
              </div>
            ) : (
              notifications.map(notification => (
                <div
                  key={notification.id}
                  className="p-3 border-b border-gray-100 hover:bg-gray-50"
                >
                  <div className="flex gap-3">
                    <div className="shrink-0 mt-0.5">
                      {getIcon(notification.type)}
                    </div>
                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium text-gray-900">{notification.title}</p>
                      <p className="text-xs text-gray-500 mt-0.5">{notification.message}</p>
                      <p className="text-xs text-gray-400 mt-1">
                        {formatTime(notification.timestamp)}
                      </p>
                    </div>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      )}
    </div>
  )
}

export function ToastContainer() {
  const { notifications, clearNotification } = useNotifications()

  return (
    <div className="fixed bottom-6 right-6 z-[200] flex flex-col gap-2">
      {notifications.slice(0, 5).map(notification => (
        <div
          key={notification.id}
          className={`min-w-[300px] p-4 rounded-lg shadow-lg flex items-start gap-3 animate-slide-in ${
            notification.type === 'success' ? 'bg-emerald-500 text-white' :
            notification.type === 'error' ? 'bg-red-500 text-white' :
            notification.type === 'warning' ? 'bg-amber-500 text-white' :
            'bg-blue-500 text-white'
          }`}
        >
          {notification.type === 'success' && <CheckCircle className="w-5 h-5 shrink-0" />}
          {notification.type === 'error' && <XCircle className="w-5 h-5 shrink-0" />}
          {notification.type === 'warning' && <AlertCircle className="w-5 h-5 shrink-0" />}
          {notification.type === 'info' && <Info className="w-5 h-5 shrink-0" />}

          <div className="flex-1">
            <p className="font-semibold">{notification.title}</p>
            <p className="text-sm opacity-90 mt-0.5">{notification.message}</p>
          </div>

          <button
            onClick={() => clearNotification(notification.id)}
            className="shrink-0 opacity-70 hover:opacity-100"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      ))}
    </div>
  )
}

function formatTime(date: Date): string {
  const now = new Date()
  const diff = now.getTime() - date.getTime()

  if (diff < 60000) return 'Just now'
  if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`
  return date.toLocaleDateString()
}

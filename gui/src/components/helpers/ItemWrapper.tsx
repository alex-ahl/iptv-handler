import React from 'react'

interface Props {
    className?: string
    display?: string
    flexDirection?: string
    alignItems?: string
    maxWidth?: string
    margin?: string
}
export const PageItemWrapper: React.FC<Props> = ({ className, children }) => {
    return <div className={className}>{children}</div>
}

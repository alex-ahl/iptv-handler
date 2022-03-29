import Navbar from 'components/Navbar'
import PageBody from 'components/PageBody'
import React from 'react'
import GlobalStyle from './styles/global'

const App: React.FC = () => (
    <>
        <GlobalStyle />
        <Navbar />
        <PageBody />
    </>
)

export default App

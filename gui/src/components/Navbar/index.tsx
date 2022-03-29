import React from 'react'
import { Container, Logo, Text, Wrapper } from './styles'

const Navbar: React.FC = () => {
    return (
        <>
            <Container>
                {/* <Logo src={logo} /> */}

                <Wrapper flexDirection="row" alignItems="center">
                    <Wrapper>
                        <Text>TEMP</Text>
                    </Wrapper>
                </Wrapper>

                {/* flag image */}
                <Wrapper flexDirection="row" alignItems="flex-start"></Wrapper>
            </Container>
        </>
    )
}

export default Navbar
